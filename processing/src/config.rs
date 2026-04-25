use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub xs_url: String,
    pub db_path:       PathBuf,
    pub cover_dir:     PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("xcalibre");
        Self {
            xs_url: "https://api.xcalibre.app".to_string(),
            db_path:       base.join("jobs.db"),
            cover_dir:     base.join("covers"),
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("xcalibre")
            .join("config.toml");

        let mut cfg: Config = if config_path.exists() {
            let text = std::fs::read_to_string(&config_path)?;
            toml::from_str::<Config>(&text)?
        } else {
            let default = Config::default();
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&config_path, toml::to_string(&default)?)?;
            default
        };

        if let Ok(url) = std::env::var("XCALIBRE_SERVER_URL") {
            cfg.xs_url = url;
        }
        if let Ok(p) = std::env::var("XCALIBRE_DB_PATH") {
            cfg.db_path = PathBuf::from(p);
        }
        Ok(cfg)
    }
}