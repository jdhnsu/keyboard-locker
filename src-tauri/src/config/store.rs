use std::fs;
use std::path::PathBuf;

use crate::locker::rules::Config;

pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new() -> Self {
        let data_dir = dirs_next().unwrap_or_else(|| PathBuf::from("."));
        let path = data_dir.join("keyboard-locker").join("config.json");
        ConfigStore { path }
    }

    pub fn with_path(path: PathBuf) -> Self {
        ConfigStore { path }
    }

    pub fn load(&self) -> Result<Config, ConfigError> {
        if !self.path.exists() {
            let config = Config::default();
            self.save(&config)?;
            return Ok(config);
        }

        let content =
            fs::read_to_string(&self.path).map_err(|e| ConfigError::ReadError(e.to_string()))?;

        let config: Config = serde_json::from_str(&content).map_err(|e| {
            if let Some(parent) = self.path.parent() {
                let bak = parent.join("config.json.bak");
                let _ = fs::write(&bak, &content);
            }
            ConfigError::ParseError(e.to_string())
        })?;

        Ok(config)
    }

    pub fn save(&self, config: &Config) -> Result<(), ConfigError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| ConfigError::WriteError(e.to_string()))?;
        }

        let tmp = self.path.with_extension("tmp");
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        fs::write(&tmp, &content).map_err(|e| ConfigError::WriteError(e.to_string()))?;
        fs::rename(&tmp, &self.path).map_err(|e| ConfigError::WriteError(e.to_string()))?;

        Ok(())
    }

    pub fn restore_defaults(&self) -> Result<Config, ConfigError> {
        let config = Config::default();
        self.save(&config)?;
        Ok(config)
    }
}

fn dirs_next() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".local").join("share"))
            })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    ReadError(String),
    ParseError(String),
    WriteError(String),
    SerializeError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ReadError(e) => write!(f, "Failed to read config: {}", e),
            ConfigError::ParseError(e) => write!(f, "Failed to parse config: {}", e),
            ConfigError::WriteError(e) => write!(f, "Failed to write config: {}", e),
            ConfigError::SerializeError(e) => write!(f, "Failed to serialize config: {}", e),
        }
    }
}
