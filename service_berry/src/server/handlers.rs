use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use hyper::header::{
    self, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
    ACCESS_CONTROL_MAX_AGE,
};
use mime::Mime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info};

use crate::geosubmit::{self, items};
use crate::server::geo_provider::{GeoProviderRequest, geo_provider_response};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PartialPayload {
    pub position: serde_json::Value,
    pub cell_towers: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

struct RequestHeaders {
    user_agent: String,
    content_type: Mime, // parsed content type
    content_length: usize,
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
            (
                StatusCode::BAD_REQUEST,
                format!("Missing or invalid {}", key),
            )
        })
}

fn validate_headers(headers: &HeaderMap) -> Result<RequestHeaders, (StatusCode, String)> {
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
        error!(
            header = "content-length",
            "Failed to parse Content-Length as number"
        );
        (
            StatusCode::LENGTH_REQUIRED,
            "Invalid Content-Length header".to_string(),
        )
    })?;
    debug!(content_length, "Parsed Content-Length header");

    Ok(RequestHeaders {
        user_agent: user_agent.to_string(),
        content_type,
        content_length,
    })
}

pub async fn submit(
    axum::Json(value): axum::Json<serde_json::Value>,
) -> Result<String, crate::error::Error> {
    let payload: PartialPayload = serde_json::from_value(value)
        .map_err(|e| crate::error::Error::Other(format!("JSON Parse Error: {}", e)))?;

    submit_payload(payload).await
}

pub async fn submit_payload(payload: PartialPayload) -> Result<String, crate::error::Error> {
    info!("Processing submission...");

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
            info!(
                "Submission taking longer than 3s; continuing in background, not checking completion status."
            );
        }
    }

    Ok(String::from("Successful"))
}

pub async fn status() -> (StatusCode, String) {
    todo!()
}

pub async fn request_options() -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("https://localhost"), // idk may need refractoring
    );
    headers.insert(
        ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("POST, GET, OPTIONS"),
    );
    headers.insert(
        ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Content-Type"),
    );
    headers.insert(ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static("86400"));

    (StatusCode::NO_CONTENT, headers).into_response()
}

pub async fn request_post(
    headers: HeaderMap,
    payload: axum::Json<serde_json::Value>,
) -> Result<Response, (StatusCode, String)> {
    let request_headers = validate_headers(&headers)?; // preliminary checks to check for correct request

    info!(
        "[Server] Request received - User-Agent: {}, Content-Type: {}, Payload: {:?}",
        request_headers.user_agent, request_headers.content_type, payload
    );

    if request_headers.content_length < 1 {
        debug!("Received keepalive request, responding with 200 OK");
        return Ok(StatusCode::OK.into_response());
    } else if !serde_json::to_string(&payload.0)
        .unwrap()
        .to_ascii_lowercase()
        .contains("macaddress")
    {
        // if payload has content but does not contain macAddress field
        error!(
            "Payload missing required field: macAddress, {}",
            serde_json::to_string(&payload.0).expect("Nothing in payload json")
        );
        return Err((
            StatusCode::BAD_REQUEST,
            "Payload missing required field: macAddress".to_string(),
        ));
    }

    let payload_data: GeoProviderRequest = serde_json::from_value(payload.0)
        .map_err(|e| crate::error::Error::Other(format!("Invalid JSON payload: {}", e)))
        .unwrap();

    let body = serde_json::to_string(&geo_provider_response(payload_data))
        .map_err(|e| crate::error::Error::Other(format!("Failed to serialize JSON: {}", e)))
        .unwrap();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap())
}

pub async fn root() -> Response {
    info!("You've reached the ServiceBerry server!");
    let html = "<!doctype html><html><head><meta charset=\"utf-8\"></head><body>hi you've reached me</body></html>";
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}
