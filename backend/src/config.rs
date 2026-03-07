use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub addr: String,
    pub port: u16,
    pub sqlite: SQLiteConfig,
    pub s3: S3Config,
    pub local_fs: LocalFsConfig,
    pub smtp: SmtpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SQLiteConfig {
    pub s3_object_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub client_id: String,
    pub client_secret: String,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalFsConfig {
    pub root_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub use_tls: bool,
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let contents =
            fs::read_to_string(path).map_err(|e| ConfigError::ReadFile(path.to_path_buf(), e))?;
        let config: Config =
            toml::from_str(&contents).map_err(|e| ConfigError::Parse(path.to_path_buf(), e))?;
        Ok(config)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file {:?}: {}", .0, .1)]
    ReadFile(std::path::PathBuf, std::io::Error),
    #[error("Failed to parse config file {:?}: {}", .0, .1)]
    Parse(std::path::PathBuf, toml::de::Error),
}
