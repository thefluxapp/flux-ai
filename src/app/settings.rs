use std::env;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct AppSettings {
    pub name: String,
    pub http: HttpSettings,
    pub ollama: OllamaSettings,
    pub nats: NATSSettings,
}

#[derive(Deserialize, Clone)]
pub struct HttpSettings {
    pub endpoint: String,
}

#[derive(Deserialize, Clone)]
pub struct NATSSettings {
    pub endpoint: String,
}

#[derive(Deserialize, Clone)]
pub struct OllamaSettings {
    pub endpoint: String,
}

impl AppSettings {
    pub fn new() -> Result<Self, ConfigError> {
        let app_dir = env::var("APP_DIR").unwrap_or_else(|_| ".".into());

        let config = Config::builder()
            .add_source(File::with_name(&format!("{}/settings/default", app_dir)))
            .add_source(File::with_name(&format!("{}/settings/local", app_dir)).required(false))
            .add_source(Environment::with_prefix("app").separator("_"))
            .build()?;

        config.try_deserialize()
    }
}
