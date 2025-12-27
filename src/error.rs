use std::fmt;

use axum::{
    Json,
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
use serde_json::json;

#[derive(Debug)]
pub enum Error {
    // Scanner errors
    BleAdapter(String),
    WifiScan(String),
    InvalidSsid(String),

    // Geosubmit errors
    Transport(String),
    HttpStatus { status: u16, body: String },
    Serialization(String),

    // Server errors
    Bind(String),

    // Config errors
    Config(String),

    // IO and serialization
    Io(std::io::Error),
    Json(serde_json::Error),

    // Other errors
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BleAdapter(msg) => write!(f, "BLE adapter error: {}", msg),
            Error::WifiScan(msg) => write!(f, "WiFi scan error: {}", msg),
            Error::InvalidSsid(msg) => write!(f, "Invalid SSID: {}", msg),
            Error::Transport(msg) => write!(f, "Transport error: {}", msg),
            Error::HttpStatus { status, body } => write!(f, "HTTP {}: {}", status, body),
            Error::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Error::Bind(msg) => write!(f, "Bind error: {}", msg),
            Error::Config(msg) => write!(f, "Config error: {}", msg),
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Json(e) => write!(f, "JSON error: {}", e),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Error::Other(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            Error::Bind(ref msg) => (StatusCode::BAD_REQUEST, msg),
            Error::BleAdapter(_) => todo!(),
            Error::WifiScan(_) => todo!(),
            Error::InvalidSsid(_) => todo!(),
            Error::Transport(_) => todo!(),
            Error::HttpStatus { status, body } => todo!(),
            Error::Serialization(_) => todo!(),
            Error::Config(_) => todo!(),
            Error::Io(error) => todo!(),
            Error::Json(error) => todo!(),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
