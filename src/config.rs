//! Config module contains the top-level config for the app.
use std::env;

use stq_http;
use stq_logging::GrayLogConfig;

use sentry_integration::SentryConfig;

use config_crate::{Config as RawConfig, ConfigError, Environment, File};

pub const ATTRIBUTE_CACHE_NAMESPACE: &'static str = "attribute";
pub const CATEGORY_CACHE_NAMESPACE: &'static str = "category";
pub const ROLES_CACHE_NAMESPACE: &'static str = "roles";

/// Basic settings - HTTP binding address and database DSN
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub client: Client,
    pub graylog: Option<GrayLogConfig>,
    pub sentry: Option<SentryConfig>,
    pub rocket_retail: Option<RocketRetail>,
    pub s3: Option<S3>,
    pub ticker: Option<Ticker>,
}

/// Common server settings
#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub host: String,
    pub port: String,
    pub database: String,
    pub elastic: String,
    pub redis: Option<String>,
    pub thread_count: usize,
    pub cache_ttl_sec: u64,
}

/// Http client settings
#[derive(Debug, Deserialize, Clone)]
pub struct Client {
    pub http_client_retries: usize,
    pub http_client_buffer_size: usize,
    pub http_timeout_ms: u64,
}

/// Http client settings
#[derive(Debug, Deserialize, Clone)]
pub struct RocketRetail {
    pub interval_s: usize,
    pub file_name: String,
    pub cluster: String,
    pub thread_count: usize,
}

/// Ticker settings
#[derive(Debug, Deserialize, Clone)]
pub struct Ticker {
    pub api_endpoint_url: String,
    pub interval_s: u64,
    pub thread_count: usize,
}

/// AWS S3 credentials
#[derive(Debug, Deserialize, Clone)]
pub struct S3 {
    pub key: String,
    pub secret: String,
    pub region: String,
    pub bucket: String,
}

/// Creates new app config struct
/// #Examples
/// ```
/// use stores_lib::config::*;
///
/// let config = Config::new();
/// ```
impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = RawConfig::new();
        s.merge(File::with_name("config/base"))?;

        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // Add in settings from the environment (with a prefix of STQ_STORES)
        s.merge(Environment::with_prefix("STQ_STORES"))?;

        s.try_into()
    }

    pub fn to_http_config(&self) -> stq_http::client::Config {
        stq_http::client::Config {
            http_client_buffer_size: self.client.http_client_buffer_size,
            http_client_retries: self.client.http_client_retries,
            timeout_duration_ms: self.client.http_timeout_ms,
        }
    }
}
