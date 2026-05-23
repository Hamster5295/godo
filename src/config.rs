use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub engine_dir: PathBuf,
    pub temp_dir: PathBuf,
    /// Cache invalidation time in seconds. Default: 10800 (3 hours)
    #[serde(default = "default_invalidate_time")]
    pub invalidate_time: u64,
    /// GitHub personal access token for API authentication. Optional but recommended to avoid rate limits.
    #[serde(default)]
    pub github_token: Option<String>,
}

fn default_invalidate_time() -> u64 {
    10800 // 3 hours
}

impl Config {
    pub fn godo_dir() -> PathBuf {
        dirs::home_dir()
            .expect("Cannot determine home directory")
            .join(".godo")
    }

    pub fn config_path() -> PathBuf {
        Self::godo_dir().join("config.toml")
    }

    pub fn manifest_path() -> PathBuf {
        Self::godo_dir().join("manifest.json")
    }

    pub fn load() -> Result<Self> {
        let godo_dir = Self::godo_dir();
        std::fs::create_dir_all(&godo_dir).context("Failed to create ~/.godo directory")?;

        let engine_dir = godo_dir.join("engine");
        std::fs::create_dir_all(&engine_dir).context("Failed to create engine directory")?;

        let config_path = Self::config_path();
        if config_path.exists() {
            let content =
                std::fs::read_to_string(&config_path).context("Failed to read config file")?;
            let config: Config = toml::from_str(&content).context("Failed to parse config file")?;
            std::fs::create_dir_all(&config.engine_dir)
                .context("Failed to create engine directory")?;
            std::fs::create_dir_all(&config.temp_dir).context("Failed to create temp directory")?;
            Ok(config)
        } else {
            let config = Config::default_config();
            config.save()?;
            Ok(config)
        }
    }

    pub fn default_config() -> Self {
        let godo_dir = Self::godo_dir();
        let temp_dir = if cfg!(target_os = "windows") {
            std::env::var("TEMP")
                .map(PathBuf::from)
                .unwrap_or_else(|_| std::env::temp_dir())
                .join("godo")
        } else {
            PathBuf::from("/tmp/godo")
        };

        Config {
            engine_dir: godo_dir.join("engine"),
            temp_dir,
            invalidate_time: default_invalidate_time(),
            github_token: None,
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        std::fs::write(&config_path, content).context("Failed to write config file")?;
        Ok(())
    }

    pub fn current_link_path(&self) -> PathBuf {
        self.engine_dir.join("current")
    }
}
