use meta_address::enums::Asset;
use meta_common::enums::CexExchange;

pub mod binance;
pub mod bitfinex;
pub mod cefi_service;
pub mod util;
pub mod model;

use bitfinex::errors::*;
use std::sync::mpsc::Sender;
use tokio::sync::mpsc::Sender as TokioSender;

use crate::binance::util::binance_asset_symbol;

pub static SYMBOL_USDT: &str = "USDT";

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

    // pub fn shutdown(&self) -> Result<()> {
    //     self.tx.send(WsMessage::Close).map_err(|e| Error::with_chain(e, "Error during shutdown"))
    // }
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

pub fn cex_currency_to_asset(_cex: CexExchange, currency: &str) -> Asset {
    currency.parse::<Asset>().unwrap()
}

pub fn get_cex_pair(cex: CexExchange, base: Asset, quote: Asset) -> String {
    match cex {
        CexExchange::BITFINEX => format!("t{}{}", base, quote),
        CexExchange::BINANCE => {
            format!("{}{}", binance_asset_symbol(&base), binance_asset_symbol(&quote))
        }
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
        assert_eq!(cex_currency_to_asset(CexExchange::BINANCE, "BNB"), Asset::BNB);
    }

    #[test]
    fn test_get_cex_pair() {
        assert_eq!(
            get_cex_pair(CexExchange::BITFINEX, Asset::ARB, Asset::USD),
            "tARBUSD".to_string()
        );
        assert_eq!(
            get_cex_pair(CexExchange::BINANCE, Asset::ARB, Asset::USD),
            "ARBUSDT".to_string()
        );
    }
}
