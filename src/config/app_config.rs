use serde::Deserialize;

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub system: SystemConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SystemConfig {
    pub adb_path: String,

    pub auto_discovery: bool,

    pub device_poll_interval_ms: u64,

    pub max_concurrent_devices: usize,

    pub action_timeout_seconds: u64,

    pub retry_count: u32,

    pub worker_queue_size: usize,
}

impl AppConfig {
    pub async fn load(path: &str) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;

        let config: Self = toml::from_str(&content).map_err(|e| AppError::Config(e.to_string()))?;

        Ok(config)
    }
}
