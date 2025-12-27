use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info};

use crate::geosubmit::{self, items};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PartialPayload {
    pub position: serde_json::Value,
    pub cell_towers: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

pub async fn process_submit_http(
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

pub async fn handle_request() -> (StatusCode, String) {
    (StatusCode::OK, "ok".to_string())
}
