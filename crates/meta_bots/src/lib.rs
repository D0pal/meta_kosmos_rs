pub mod forked_db;
pub mod mev_bots;

use async_trait::async_trait;
use config::{Config, ConfigError, File};
use meta_cefi::cefi_service::AccessKey;
use meta_common::enums::{CexExchange, DexExchange, Network,  RpcProvider};
use meta_address::enums::Asset;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::env;
use std::{path::PathBuf, result::Result, str::FromStr};
use tracing::Level;

use meta_tracing::TraceConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigLog {
    pub file_name_prefix: String,
    pub dir: String,
    pub level: String,
    pub flame: bool,
    pub console: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigChain {
    pub network: Option<Network>,
    pub provider: Option<RpcProvider>,
    pub dexs: Option<Vec<DexExchange>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigProvider {
    pub ws_interval_milli: Option<u64>,
    pub provider: Option<RpcProvider>
}

#[derive(Debug, Deserialize)]
pub struct ConfigRds {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigAccount {
    pub private_key_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub log: ConfigLog,
    // pub rds: ConfigRds,
}

// #[async_trait]
impl AppConfig {
    pub fn load(dir: &str) -> Result<Config, ConfigError> {
        let env = env::var("ENV").unwrap_or("dev".into());
        Config::builder()
            .add_source(File::with_name(&format!("{}/default", dir)))
            .add_source(File::with_name(&format!("{}/{}", dir, env)).required(false))
            .add_source(File::with_name(&format!("{}/local", dir)).required(false))
            .add_source(config::Environment::with_prefix("META"))
            .build()
    }
    pub fn try_new() -> Result<Self, ConfigError> {
        let config = Self::load("config")?;
        config.try_deserialize()
    }
}

impl From<ConfigLog> for TraceConfig {
    fn from(config_log: ConfigLog) -> Self {
        let level = Level::from_str(&config_log.level)
            .expect(&format!("converting level: {} error", &config_log.level));
        TraceConfig {
            file_name_prefix: config_log.file_name_prefix,
            dir: config_log.dir,
            level: level,
            flame: config_log.flame,
            console: config_log.console,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupyterConfig {
    pub log: ConfigLog,
    pub chain: ConfigChain,
    pub provider: ConfigProvider,
    pub accounts: ConfigAccount,
}

impl JupyterConfig {
    pub fn load(dir: &str) -> Result<Config, ConfigError> {
        let env = env::var("ENV").unwrap_or("default".into());
        Config::builder()
            // .add_source(File::with_name(&format!("{}/default", dir)))
            .add_source(File::with_name(&format!("{}/{}", dir, env)).required(false))
            .add_source(File::with_name(&format!("{}/local", dir)).required(false))
            .add_source(config::Environment::with_prefix("META_JUPYTER"))
            .build()
    }
    pub fn try_new() -> Result<Self, ConfigError> {
        let config = Self::load("config/jupyter")?;
        config.try_deserialize()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct VenusConfig {
    pub network: Network,
    pub dex: DexExchange,
    pub cex: CexExchange,
    pub base_asset: Asset,
    pub quote_asset: Asset,
    pub base_asset_quote_amt: Decimal,
    pub log: ConfigLog,
    pub provider: ConfigProvider,
    pub account: ConfigAccount,
    pub bitfinex: Option<AccessKey>
}

impl VenusConfig {
    pub fn load(dir: &str) -> Result<Config, ConfigError> {
        let env = env::var("ENV").unwrap_or("default".into());
        Config::builder()
            // .add_source(File::with_name(&format!("{}/default", dir)))
            .add_source(File::with_name(&format!("{}/{}", dir, env)).required(false))
            .add_source(config::Environment::with_prefix("META_VENUS"))
            .build()
    }
    pub fn try_new() -> Result<Self, ConfigError> {
        let config = Self::load("config/venus")?;
        config.try_deserialize()
    }
}
