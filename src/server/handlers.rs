use axum::http::{HeaderMap, StatusCode, HeaderValue};
use axum::response::{IntoResponse, Response};
use hyper::header::{ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_MAX_AGE, CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT};
use mime::Mime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info};

use crate::geosubmit::{self, items};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PartialPayload {
    pub position: serde_json::Value,
    pub cell_towers: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

pub struct RequestHeaders {
    pub user_agent: String,
    pub content_type: Mime, // parsed content type
    pub content_length: usize,
}

fn get_header<'a>(
    headers: &'a HeaderMap,
    key: &'static str,
) -> Result<&'a str, (StatusCode, String)> {
    headers
        .get(key)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            error!(header = key, "Missing or invalid header");
            (StatusCode::BAD_REQUEST, format!("Missing or invalid {}", key))
        })
}


pub fn validate_headers(headers: &HeaderMap) -> Result<RequestHeaders, (StatusCode, String)> {
    // User-Agent
    let user_agent = get_header(headers, "user-agent")?;
    debug!(user_agent = %user_agent, "Parsed User-Agent header");

    // Content-Type
    let content_type_str = get_header(headers, "content-type")?;
    let content_type = content_type_str.parse::<Mime>().map_err(|e| {
        error!(header = "content-type", error = %e, "Failed to parse Content-Type as MIME type");
        (
            StatusCode::BAD_REQUEST,
            "Invalid MIME type in Content-Type header".to_string(),
        )
    })?;
    debug!(content_type = %content_type, "Parsed Content-Type header");

    // Content-Length
    let content_length_str = get_header(headers, "content-length")?;
    let content_length = content_length_str.parse::<usize>().map_err(|_| {
        error!(header = "content-length", "Failed to parse Content-Length as number");
        (StatusCode::LENGTH_REQUIRED, "Invalid Content-Length header".to_string())
    })?;
    debug!(content_length, "Parsed Content-Length header");

    Ok(RequestHeaders {
        user_agent: user_agent.to_string(),
        content_type,
        content_length,
    })
}

pub async fn process_submit_https(
    axum::Json(value): axum::Json<serde_json::Value>,
) -> Result<String, crate::error::Error> {
    let payload: PartialPayload = serde_json::from_value(value)
        .map_err(|e| crate::error::Error::Other(format!("JSON Parse Error: {}", e)))?;

    process_submit(payload).await
}

pub async fn process_submit(payload: PartialPayload) -> Result<String, crate::error::Error> {
    info!("[Server] Processing submission...");

    let geo_items: items = geosubmit::assemble_geo_payload(payload.position, payload.cell_towers)
        .await
        .map_err(|e| crate::error::Error::Other(format!("Assembly Error: {}", e)))?;

    let handle = tokio::spawn(async move { geosubmit::submit_geo_payload(geo_items).await });

    match timeout(Duration::from_secs(3), handle).await {
        Ok(join_result) => match join_result {
            Ok(Ok(())) => {
                info!("Successfully sent geolocation data to service");
            }
            Ok(Err(e)) => {
                error!("Geosubmit network error: {}", e);
            }
            Err(join_err) => {
                error!("Submission task panicked: {:?}", join_err);
            }
        },
        Err(_) => {
            info!("Submission taking longer than 3s; continuing in background.");
        }
    }

    Ok(String::from("Successful"))
}

pub async fn handle_status() -> (StatusCode, String) {
    (StatusCode::OK, "ok".to_string())
}

pub async fn handle_request_options() -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("https://localhost"),
    );
    headers.insert(
        ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("POST, GET, OPTIONS"),
    );
    headers.insert(
        ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Content-Type"),
    );
    headers.insert(
        ACCESS_CONTROL_MAX_AGE,
        HeaderValue::from_static("86400"),
    );
    
    (StatusCode::NO_CONTENT, headers).into_response()
}

pub async fn handle_request(
    headers: HeaderMap,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> Result<Response, (StatusCode, String)> {
    let request_headers = validate_headers(&headers)?; // preliminary checks to check for correct request

    info!(
        "[Server] Request received - User-Agent: {}, Content-Type: {}, Payload: {:?}",
        request_headers.user_agent, request_headers.content_type, payload
    );

    Ok((StatusCode::OK, "Request received").into_response())
}
