use crate::bitfinex::{auth, errors::*};
use error_chain::bail;
use reqwest::{
    blocking::{Client as HttpClient, Response},
    header,
    header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, USER_AGENT},
    StatusCode,
};
use serde::Serialize;
use std::io::Read;
use tracing::debug;

static API1_HOST: &str = "https://api.bitfinex.com/v2/";
static API_SIGNATURE_PATH: &str = "/api/v2/auth/r/";
static API_SIGNATURE_PATH_ORDER: &str = "/api/v2/auth/w/";
static NO_PARAMS: &[(); 0] = &[];

#[derive(Clone)]
pub struct Client {
    api_key: String,
    secret_key: String,
    http_client: HttpClient,
}

impl Client {
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        let mut default_header = header::HeaderMap::new();
        default_header.insert("Content-Type", header::HeaderValue::from_static("application/json"));
        let client = reqwest::blocking::ClientBuilder::new()
            // .http2_keep_alive_while_idle(true)
            .default_headers(default_header)
            .build()
            .expect("unable to build http client");

        Client {
            api_key: api_key.unwrap_or("".into()),
            secret_key: secret_key.unwrap_or("".into()),
            http_client: client,
        }
    }

    pub fn get(&self, endpoint: String, request: String) -> Result<String> {
        let mut url: String = format!("{}{}", API1_HOST, endpoint);
        if !request.is_empty() {
            url.push_str(format!("?{}", request).as_str());
        }

        let response = reqwest::blocking::get(url.as_str())?;

        self.handler(response)
    }

    pub fn post_signed(&self, request: String, payload: String) -> Result<String> {
        self.post_signed_params(request, payload, NO_PARAMS, false)
    }

    pub fn post_signed_order(&self, request: String, payload: String) -> Result<String> {
        self.post_signed_params(request, payload, NO_PARAMS, true)
    }

    pub fn post_signed_params<P: Serialize + ?Sized>(
        &self,
        request: String,
        payload: String,
        params: &P,
        is_order_request: bool,
    ) -> Result<String> {
        let url = match is_order_request {
            true => format!("{}auth/w/{}", API1_HOST, request),
            _ => format!("{}auth/r/{}", API1_HOST, request),
        };

        let api_signature_path =
            if is_order_request { API_SIGNATURE_PATH_ORDER } else { API_SIGNATURE_PATH };

        debug!("{} {} {}", url, api_signature_path, payload);
        // let client = reqwest::blocking::Client::new();
        let response = self
            .http_client
            .post(url.as_str())
            .headers(self.build_headers(
                request,
                payload.clone(),
                api_signature_path.to_string(),
            )?)
            .body(payload)
            .query(params)
            .send()?;

        self.handler(response)
    }

    fn build_headers(
        &self,
        request: String,
        payload: String,
        api_signature_path: String,
    ) -> Result<HeaderMap> {
        let nonce: String = auth::generate_nonce()?;
        let signature_path: String =
            format!("{}{}{}{}", api_signature_path, request, nonce, payload);

        let signature = auth::sign_payload(self.secret_key.as_bytes(), signature_path.as_bytes())?;

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("bitfinex-rs"));
        headers
            .insert(HeaderName::from_static("bfx-nonce"), HeaderValue::from_str(nonce.as_str())?);
        headers.insert(
            HeaderName::from_static("bfx-apikey"),
            HeaderValue::from_str(self.api_key.as_str())?,
        );
        headers.insert(
            HeaderName::from_static("bfx-signature"),
            HeaderValue::from_str(signature.as_str())?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        Ok(headers)
    }

    fn handler(&self, mut response: Response) -> Result<String> {
        match response.status() {
            StatusCode::OK => {
                let mut body = String::new();
                response.read_to_string(&mut body)?;
                Ok(body)
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                bail!("Internal Server Error");
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                bail!("Service Unavailable");
            }
            StatusCode::UNAUTHORIZED => {
                bail!("Unauthorized");
            }
            StatusCode::BAD_REQUEST => {
                bail!(format!("Bad Request: {:?}", response));
            }
            s => {
                bail!(format!("Received response: {:?}", s));
            }
        }
    }
}
