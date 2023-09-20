//! TODO: use a macro to auto serialize to UPPERCASE
use serde::Serialize;
use strum::{Display};

#[derive(Copy, Clone, Debug, Display, Serialize)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Side {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

#[derive(Copy, Clone, Debug, Display, Serialize)]
#[strum(serialize_all = "UPPERCASE")]
pub enum TimeInForce {
    #[serde(rename = "GTC")]
    Gtc, // Good-Till-Cancel
    #[serde(rename = "IOC")]
    Ioc, // Immediate-Or-Cancel, the order will attempt to execute all or part of it immediately at the price and quantity available, then cancel any remaining, unfilled part of the order. If no quantity is available at the chosen price when you place the order, it will be canceled immediately. Please note that Iceberg orders are not supported.
    #[serde(rename = "FOK")]
    Fok, // Fill-Or-Kill, the order is instructed to execute in full immediately (filled), otherwise it will be canceled (killed). Please note that Iceberg orders are not supported.
}

#[derive(Copy, Debug, Clone, Display, Serialize)]
#[strum(serialize_all = "UPPERCASE")]
pub enum NewOrderResponseType {
    #[serde(rename = "ACK")]
    Ack,
    #[serde(rename = "RESULT")]
    Result,
    #[serde(rename = "FULL")]
    Full,
}

#[derive(Copy, Clone, Display, Serialize)]
pub enum CancelReplaceMode {
    #[serde(rename = "STOP_ON_FAILURE")]
    StopOnFailure,
    #[serde(rename = "ALLOW_FAILURE")]
    AllowFailure,
}
