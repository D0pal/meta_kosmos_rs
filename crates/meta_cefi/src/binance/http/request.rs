use std::collections::BTreeMap;

use serde_json::Value;

use crate::binance::http::{Credentials, Method};

#[derive(PartialEq, Eq, Debug)]
pub struct Request {
    // pub(crate) body_json: Option<Value>,
    pub(crate) method: Method,
    pub(crate) path: String,
    pub(crate) params: BTreeMap<String, String>,
    pub(crate) credentials: Option<Credentials>,
    pub(crate) sign: bool,
}

impl Request {
    pub fn method(&self) -> &Method {
        &self.method
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn params(&self) -> &BTreeMap<String, String> {
        &self.params
    }
    pub fn credentials(&self) -> &Option<Credentials> {
        &self.credentials
    }
    pub fn sign(&self) -> &bool {
        &self.sign
    }

    pub fn get_payload_to_sign(&self) -> String {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());

        let iter = self.params.iter();
        for (key, val) in iter {
            serializer.append_pair(key, val);
        }
        let mut query_string = serializer.finish();
        query_string
    }
}

/// /// API HTTP Request
///
/// A low-level request builder for API integration
/// decoupled from any specific underlying HTTP library.
pub struct RequestBuilder {
    method: Method,
    path: String,
    // body_json: Option<serde_json::Value>,
    params: BTreeMap<String, String>,
    credentials: Option<Credentials>,
    sign: bool,
}

impl RequestBuilder {
    pub fn new(method: Method, path: &str) -> Self {
        Self {
            // body_json: None,
            method,
            path: path.to_owned(),
            params: BTreeMap::new(),
            credentials: None,
            sign: false,
        }
    }

    /// Append `params` to the request's query string. Parameters may
    /// share the same key, and will result in a query string with one or
    /// more duplicated query parameter keys.
    pub fn params<'a>(mut self, params: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        self.params
            .extend(params.into_iter().map(|param| (param.0.to_owned(), param.1.to_owned())));

        self
    }

    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);

        self
    }

    pub fn sign(mut self) -> Self {
        self.sign = true;

        self
    }
}

impl From<RequestBuilder> for Request {
    fn from(builder: RequestBuilder) -> Request {
        Request {
            // body_json: builder.body_json,
            method: builder.method,
            path: builder.path,
            params: builder.params,
            credentials: builder.credentials,
            sign: builder.sign,
        }
    }
}
