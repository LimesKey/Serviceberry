//! HTTP request handlers for server endpoints

use axum::{Json, http::StatusCode};
use std::time::Duration;
use tokio::time::timeout;
use serde::Deserialize;
use std::collections::HashMap;

use crate::geosubmit::{self, items};

/// Partial payload from client
#[derive(Deserialize)]
pub struct PartialPayload { // incoming json structure from client (mobile) device 
    pub position: serde_json::Value, // requires
    pub cell_towers: Option<serde_json::Value>, // optional
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>, // catch-all for any other fields
}

/// Handle /submit endpoint
pub async fn process_submit(
    Json(payload): Json<serde_json::Value>,
) -> Result<String, StatusCode> {
    println!("[Server] /submit request received");

    println!("\n================ RECEIVED JSON =================");
    println!("{}", &payload);
    println!("============================================\n");

    let gps_response: PartialPayload = serde_json::from_value(payload)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let response: items = geosubmit::assemble_geo_payload(gps_response.position, gps_response.cell_towers)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let handle = tokio::spawn(async move {
        geosubmit::submit_geo_payload(response.clone()).await
    });

    match timeout(Duration::from_secs(3), handle).await {
        Ok(join_result) => match join_result {
            Ok(Ok(())) => {
                println!("Successfully sent geolocation data");
            }
            Ok(Err(e)) => {
                eprintln!("Geosubmit error: {}", e);
            }
            Err(join_err) => {
                eprintln!("Task panicked: {:?}", join_err);
            }
        },

        Err(_) => {
            tracing::debug!("Request took too long, not waiting for status...");
        }
    }

    Ok(String::from("Successful"))
}

/// Handle /status endpoint
pub async fn handle_status() -> Result<String, StatusCode> {
    Ok(String::from("ok"))
}

/// Handle /request endpoint
pub async fn handle_request() -> Result<String, StatusCode> {
    Ok(String::from("ok"))
}
