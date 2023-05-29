use std::env;
use std::{result::Result, str::FromStr};

// use actix_web::{App, web};
use async_trait::async_trait;
use config::{Config, ConfigError, File};
use serde::Deserialize;
use tracing::Level;

use meta_tracing::TraceConfig;

// use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ConfigLog {
    pub file_name_prefix: String,
    pub dir: String,
    pub level: String,
    pub flame: bool,
    pub console: bool,
}

#[derive(Debug, Deserialize)]
pub struct ConfigRds {
    pub url: String,
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

#[derive(Debug, Deserialize)]
pub struct JupyterConfig {
    pub log: ConfigLog,
    // pub rds: ConfigRds,
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

