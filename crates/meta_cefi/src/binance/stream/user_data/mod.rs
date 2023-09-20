pub mod close_listen_key;
pub mod new_listen_key;
pub mod renew_listen_key;

use crate::binance::stream::Stream;

use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenKeyResult {
    pub listen_key: String,
}

use self::{
    close_listen_key::CloseListenKey, new_listen_key::NewListenKey,
    renew_listen_key::RenewListenKey,
};

pub fn new_listen_key() -> NewListenKey {
    NewListenKey::new()
}

pub fn renew_listen_key(listen_key: &str) -> RenewListenKey {
    RenewListenKey::new(listen_key)
}

pub fn close_listen_key(listen_key: &str) -> CloseListenKey {
    CloseListenKey::new(listen_key)
}

/// User Data Stream.
///
/// A User Data Stream listenKey is valid for 60 minutes after creation.
///
/// Possible Updates:
///
/// * `outboundAccountPosition` is sent any time an account balance has
/// changed and contains the assets that were possibly changed by
/// the event that generated the balance change.
///
/// * `balanceUpdate` occurs during the following: Deposits or
/// withdrawals from the account; Transfer of funds between
/// accounts (e.g. Spot to Margin).
///
/// * `executionReport` occurs when an order is updated. If the order is
/// an OCO, an event will be displayed named `ListStatus` in addition
/// to the `executionReport` event.
///
/// [API Documentation](https://binance-docs.github.io/apidocs/spot/en/#user-data-streams)
pub struct UserDataStream {
    listen_key: String,
}

impl UserDataStream {
    pub fn new(listen_key: &str) -> Self {
        Self { listen_key: listen_key.to_owned() }
    }
}

impl From<UserDataStream> for Stream {
    /// Returns stream name as `<listen_key>`
    fn from(stream: UserDataStream) -> Stream {
        Stream::new(&stream.listen_key)
    }
}

// user data example
// 2023-09-19T08:13:41.763664Z DEBUG binance_ws_async: {"e":"executionReport","E":1695111221723,"s":"ARBUSDT","c":"web_21dc0dcdab3a4ccaa90536e6868555bc","S":"BUY","o":"MARKET","f":"GTC","q":"5.90000000","p":"0.00000000","P":"0.00000000","F":"0.00000000","g":-1,"C":"","x":"NEW","X":"NEW","r":"NONE","i":492066722,"l":"0.00000000","z":"0.00000000","L":"0.00000000","n":"0","N":null,"T":1695111221722,"t":-1,"I":1018705623,"w":true,"m":false,"M":false,"O":1695111221722,"Z":"0.00000000","Y":"0.00000000","Q":"5.00000000","W":1695111221722,"V":"NONE"}
// 2023-09-19T08:13:41.765592Z DEBUG binance_ws_async: {"e":"executionReport","E":1695111221723,"s":"ARBUSDT","c":"web_21dc0dcdab3a4ccaa90536e6868555bc","S":"BUY","o":"MARKET","f":"GTC","q":"5.90000000","p":"0.00000000","P":"0.00000000","F":"0.00000000","g":-1,"C":"","x":"TRADE","X":"FILLED","r":"NONE","i":492066722,"l":"5.90000000","z":"5.90000000","L":"0.83900000","n":"0.00001706","N":"BNB","T":1695111221722,"t":34491699,"I":1018705624,"w":false,"m":false,"M":true,"O":1695111221722,"Z":"4.95010000","Y":"4.95010000","Q":"5.00000000","W":1695111221722,"V":"NONE"}
//
