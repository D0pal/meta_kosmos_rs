#![allow(clippy::needless_doctest_main)]

pub mod account;
pub mod api;
pub mod client;
pub mod config;
pub mod constants;
pub mod errors;
pub mod general;
pub mod handler;
pub mod http;
pub mod hyper;
pub mod market;
pub mod model;
pub mod stream;
pub mod trade;
pub mod util;
pub mod websockets;
pub mod websockets_tokio;

pub(crate) const VERSION: &str = "1.0.1";
