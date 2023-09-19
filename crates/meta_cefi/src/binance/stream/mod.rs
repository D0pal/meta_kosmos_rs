// pub mod new_listen_key;
pub mod market;
pub mod user_data;

use std::fmt;

// pub mod close_listen_key;
// pub mod renew_listen_key;

// use close_listen_key::CloseListenKey;
// use new_listen_key::NewListenKey;
// use renew_listen_key::RenewListenKey;

// pub fn renew_listen_key(listen_key: &str) -> RenewListenKey {
//     RenewListenKey::new(listen_key)
// }

// pub fn close_listen_key(listen_key: &str) -> CloseListenKey {
//     CloseListenKey::new(listen_key)
// }

/// Websocket stream.
///
/// The `Stream` trait is a simplified interface for Binance approved
/// websocket streams.
pub struct Stream {
    stream_name: String,
}

impl Stream {
    pub fn new(stream_name: &str) -> Self {
        Self { stream_name: stream_name.to_owned() }
    }

    pub fn as_str(&self) -> &str {
        &self.stream_name
    }
}

impl fmt::Display for Stream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.stream_name)
    }
}
