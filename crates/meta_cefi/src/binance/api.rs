use crate::binance::{client::Client, config::Config, general::General, market::Market};

use super::account::Account;

#[allow(clippy::all)]
pub enum API {
    Spot(Spot),
}

/// Endpoint for production and test orders.
///
/// Orders issued to test are validated, but not sent into the matching engine.
pub enum Spot {
    Ping,
    Time,
    ExchangeInfo,
    Depth,
    Trades,
    HistoricalTrades,
    AggTrades,
    Klines,
    AvgPrice,
    Ticker24hr,
    Price,
    BookTicker,
    Order,
    OrderTest,
    OpenOrders,
    AllOrders,
    Oco,
    OrderList,
    AllOrderList,
    OpenOrderList,
    Account,
    MyTrades,
    UserDataStream,
}

impl From<API> for String {
    fn from(item: API) -> Self {
        String::from(match item {
            API::Spot(route) => match route {
                Spot::Ping => "/api/v3/ping",
                Spot::Time => "/api/v3/time",
                Spot::ExchangeInfo => "/api/v3/exchangeInfo",
                Spot::Depth => "/api/v3/depth",
                Spot::Trades => "/api/v3/trades",
                Spot::HistoricalTrades => "/api/v3/historicalTrades",
                Spot::AggTrades => "/api/v3/aggTrades",
                Spot::Klines => "/api/v3/klines",
                Spot::AvgPrice => "/api/v3/avgPrice",
                Spot::Ticker24hr => "/api/v3/ticker/24hr",
                Spot::Price => "/api/v3/ticker/price",
                Spot::BookTicker => "/api/v3/ticker/bookTicker",
                Spot::Order => "/api/v3/order",
                Spot::OrderTest => "/api/v3/order/test",
                Spot::OpenOrders => "/api/v3/openOrders",
                Spot::AllOrders => "/api/v3/allOrders",
                Spot::Oco => "/api/v3/order/oco",
                Spot::OrderList => "/api/v3/orderList",
                Spot::AllOrderList => "/api/v3/allOrderList",
                Spot::OpenOrderList => "/api/v3/openOrderList",
                Spot::Account => "/api/v3/account",
                Spot::MyTrades => "/api/v3/myTrades",
                Spot::UserDataStream => "/api/v3/userDataStream",
            },
        })
    }
}

pub trait Binance {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Self;
    fn new_with_config(
        api_key: Option<String>,
        secret_key: Option<String>,
        config: &Config,
    ) -> Self;
}

impl Binance for General {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> General {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    fn new_with_config(
        api_key: Option<String>,
        secret_key: Option<String>,
        config: &Config,
    ) -> General {
        General { client: Client::new(api_key, secret_key, config.rest_api_endpoint.clone()) }
    }
}

impl Binance for Market {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Market {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    fn new_with_config(
        api_key: Option<String>,
        secret_key: Option<String>,
        config: &Config,
    ) -> Market {
        Market {
            client: Client::new(api_key, secret_key, config.rest_api_endpoint.clone()),
            recv_window: config.recv_window,
        }
    }
}

impl Binance for Account {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Account {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    fn new_with_config(
        api_key: Option<String>, secret_key: Option<String>, config: &Config,
    ) -> Account {
        Account {
            client: Client::new(api_key, secret_key, config.rest_api_endpoint.clone()),
            recv_window: config.recv_window,
        }
    }
}