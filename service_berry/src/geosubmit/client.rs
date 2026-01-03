//! HTTP client for submitting geosubmit payloads

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_tracing::TracingMiddleware;
use tracing::debug;

use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::Instant;

use crate::config::{APP_USER_AGENT, GEOSUBMIT_ENDPOINT};
use crate::error::{Error, Result};
use crate::scanner::{bluetooth, wifi};

use super::payload::{GeosubmitRequest, items};

/// Assemble geolocation payload from current scans
pub async fn assemble_geo_payload(
    position: serde_json::Value,
    cell_towers: Option<serde_json::Value>,
) -> Result<items> {
    let config = crate::config::get_config();

    let position: crate::geosubmit::payload::Position =
        serde_json::from_value(position).map_err(|e| Error::Serialization(e.to_string()))?;

    let cell_towers: Option<Vec<crate::geosubmit::payload::CellTower>> = match cell_towers {
        Some(ct_value) => Some(
            serde_json::from_value(ct_value).map_err(|e| Error::Serialization(e.to_string()))?,
        ),
        None => None,
    };

    debug!("Mobile Device Request: {:?}", position);

    let wifi_start = Instant::now();
    let ble_start = Instant::now();

    let ifindex = config.interface.index;

    let (wifi_result, ble) = tokio::join!(
        // run simultaneously
        wifi::scan(ifindex),
        bluetooth::fetch_ble_devices()
    );

    let wifi = wifi_result.map_err(|e| Error::WifiScan(e.to_string()))?;


    let wifi_duration = wifi_start.elapsed();
    let ble_duration = ble_start.elapsed();
    tracing::debug!("WiFi scan duration: {:?}", wifi_duration);
    tracing::debug!("BLE scan duration: {:?}", ble_duration);

    let unix_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let payload = items {
        timestamp: unix_epoch,
        position,
        wifiAccessPoints: wifi,
        bluetoothBeacons: ble,
        CellTowers: cell_towers,
    };

    payload.save_cache().unwrap();

    Ok(payload)
}

/// Submit geolocation payload to the geosubmit API
pub async fn submit_geo_payload(payload: items) -> Result<()> {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(5);
    let https_client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| Error::Transport(e.to_string()))?;

    let request_body = GeosubmitRequest {
        items: vec![payload],
    };

    debug!(
        "Request JSON Body: {:?}",
        serde_json::to_string_pretty(&request_body).unwrap_or_default()
    );

    let client: ClientWithMiddleware = ClientBuilder::new(https_client.clone())
        .with(TracingMiddleware::default())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let req = https_client
        .post(GEOSUBMIT_ENDPOINT)
        .json(&request_body)
        .build()
        .map_err(|e| Error::Transport(e.to_string()))?;

    let res = client
        .execute(req)
        .await
        .map_err(|e| Error::Transport(e.to_string()))?;

    let status = res.status();
    let body = res.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(Error::HttpStatus {
            status: status.as_u16(),
            body,
        });
    }

    tracing::info!("BeaconDB returned response status: {}", status);

    Ok(())
}
