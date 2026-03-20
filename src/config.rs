use crate::errors::SmError;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub region: Option<String>,
    pub profile: Option<String>,
    /// Default output format (env, json, yaml, csv, stdout)
    #[allow(dead_code)]
    pub format: Option<String>,
}

pub fn load_config() -> Result<Config, SmError> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".sm2env")
}
