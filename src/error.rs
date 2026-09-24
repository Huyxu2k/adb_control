use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("ADB error: {0}")]
    Adb(String),

    #[error("ADB command failed: {0}")]
    AdbCommand(String),

    #[error("device not found: {0}")]
    DeviceNotFound(String),

    #[error("device offline: {0}")]
    DeviceOffline(String),

    #[error("UI node not found")]
    ElementNotFound,

    #[error("UI parse error: {0}")]
    UiParse(String),

    #[error("invalid selector")]
    InvalidSelector,

    #[error("action failed: {0}")]
    ActionFailed(String),

    #[error("timeout")]
    Timeout,

    #[error("configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;