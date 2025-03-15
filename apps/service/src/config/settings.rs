use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Redis {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Storage {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MarkdownService {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Api {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Scraper {
    pub default_user_agent: String,
    pub default_delay_ms: u32,
    pub max_concurrent_requests: u32,
    pub max_retries: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Scheduler {
    pub enabled: bool,
    pub check_interval_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: Database,
    pub redis: Redis,
    pub storage: Storage,
    pub markdown_service: MarkdownService,
    pub api: Api,
    pub scraper: Scraper,
    pub scheduler: Scheduler,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Load .env file if it exists
        dotenv::dotenv().ok();

        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        let config_dir = env::var("CONFIG_DIR").unwrap_or_else(|_| "./config".into());

        let s = Config::builder()
            // Start with default settings
            .add_source(File::with_name(&format!("{}/default", config_dir)).required(false))
            // Add mode-specific settings
            .add_source(File::with_name(&format!("{}/{}", config_dir, run_mode)).required(false))
            // Add local settings
            .add_source(File::with_name(&format!("{}/local", config_dir)).required(false))
            // Add environment variables with prefix "APP_"
            .add_source(Environment::with_prefix("APP").separator("_"))
            .build()?;

        s.try_deserialize()
    }
} 