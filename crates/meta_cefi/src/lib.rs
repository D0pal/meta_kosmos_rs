use meta_address::enums::Asset;
use meta_common::enums::CexExchange;

pub mod binance;
pub mod bitfinex;
pub mod cefi_service;
pub mod util;

use bitfinex::errors::*;
use std::sync::mpsc::Sender;
use tokio::sync::mpsc::Sender as TokioSender;

#[derive(Debug, Clone)]
pub enum MessageChannel {
    Stream, // subscribe order book
    Trade,  // submit order
}

#[derive(Debug, Clone)]
pub enum WsMessage {
    Close,
    Text(MessageChannel, String),
}

#[derive(Clone)]
pub(crate) struct WsBackendSender {
    tx: Sender<WsMessage>,
}

impl WsBackendSender {
    pub fn send(&self, ty: MessageChannel, raw: &str) -> Result<()> {
        self.tx
            .send(WsMessage::Text(ty, raw.to_string()))
            .map_err(|e| Error::with_chain(e, "Not able to send a message"))?;
        Ok(())
    }

    pub fn shutdown(&self) -> Result<()> {
        self.tx.send(WsMessage::Close).map_err(|e| Error::with_chain(e, "Error during shutdown"))
    }
}

#[derive(Clone)]
pub(crate) struct WsBackendSenderAsync {
    tx: TokioSender<WsMessage>,
}

impl WsBackendSenderAsync {
    pub async fn send(&self, ty: MessageChannel, raw: &str) -> Result<()> {
        self.tx
            .send(WsMessage::Text(ty, raw.to_string()))
            .await
            .map_err(|e| Error::with_chain(e, "Not able to send a message"))?;
        Ok(())
    }
}

pub fn cex_currency_to_asset(cex: CexExchange, currency: &str) -> Asset {
    match cex {
        _ => currency.parse::<Asset>().unwrap(),
    }
}

pub fn get_cex_pair(cex: CexExchange, base: Asset, quote: Asset) -> String {
    match cex {
        CexExchange::BITFINEX => format!("t{:?}{:?}", base, quote),
        _ => format!("{:?}{:?}", base, quote),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use meta_address::enums::Asset;
    #[test]
    fn test_cex_currency_to_asset() {
        assert_eq!(cex_currency_to_asset(CexExchange::BITFINEX, "ARB"), Asset::ARB);
        assert_eq!(cex_currency_to_asset(CexExchange::BITFINEX, "USD"), Asset::USD);
    }

    #[test]
    fn test_get_cex_pair() {
        assert_eq!(
            get_cex_pair(CexExchange::BINANCE, Asset::ARB, Asset::USDT),
            "ARBUSDT".to_string()
        );
    }
}
