#![allow(clippy::needless_doctest_main)]

pub mod api;
pub mod account;
pub mod client;
pub mod config;
pub mod errors;
pub mod general;
pub mod market;
pub mod model;
pub mod util;
pub mod websockets;
pub mod websockets_tokio;
pub mod trade;
pub mod http;
pub mod handler;
pub mod hyper;
pub mod stream;
pub mod constants;

pub(crate) const VERSION: &str = "1.0.1";