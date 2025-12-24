// For Geosubmit Version 2: /v2/geosubmit
// https://ichnaea.readthedocs.io/en/latest/api/geosubmit2.html

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_tracing::TracingMiddleware;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::{GEOSUBMIT_ENDPOINT, PartialPayload, adapters};
use adapters::bluetooth::BleDevice;
use adapters::wifi::WifiBssid;
use std::time::{SystemTime, UNIX_EPOCH};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Serialize, Debug, Deserialize, Clone)]
#[allow(nonstandard_style)]
pub struct items {
    timestamp: u128, // since last unix epoch in milliseconds
    position: Position,
    bluetoothBeacons: Vec<BleDevice>,
    wifiAccessPoints: Vec<WifiBssid>,
    CellTowers: Option<Vec<CellTower>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct Position {
    latitude: f64,
    longitude: f64,
    accuracy: f64,
    altitude: f64,
    altitudeAccuracy: f64,
    heading: f64,
    speed: f64,
    source: String,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct CellTower {
    pub radioType: RadioType,   // "gsm", "wcdma", or "lte"
    pub mobileCountryCode: u16, // MCC
    pub mobileNetworkCode: u16, // MNC
    pub locationAreaCode: u32,  // LAC (GSM/WCDMA) or TAC (LTE)
    pub cellId: u32,            // Cell Identity
    pub age: Option<u32>,       // ms since last seen
    pub asu: Option<u8>,        // Arbitrary Strength Unit
}

#[derive(Serialize, Debug, Deserialize, Clone)]
enum RadioType {
    GSM,
    WCDMA,
    LTE,
}

pub async fn assemble_geo_payload(gps_pos: PartialPayload) -> Result<items, serde_json::Error> {
    let position: Position = serde_json::from_value(gps_pos.position)?;
    let cell_towers: Option<Vec<CellTower>> = match gps_pos.cell_towers {
        Some(ct_value) => Some(serde_json::from_value(ct_value)?),
        None => None,
    };

    let wifi_start = Instant::now();
    let ble_start = Instant::now();

    let (wifi, ble) = tokio::join!(
        // run simultaneously
        adapters::wifi::fetch_wifi_stats(),
        adapters::bluetooth::fetch_ble_devices()
    );

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

    Ok(payload)
}

// this took horribly long to write, fuck reqwest middleware
impl items {
    pub async fn submit_geo_payload(payload: items) -> Result<(), reqwest_middleware::Error> {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(5);
        let http_client = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        // Wrap it in middleware
        let client: ClientWithMiddleware = ClientBuilder::new(http_client.clone())
            .with(TracingMiddleware::default())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        // Build the request using reqwest directly
        let req = http_client
            .post(GEOSUBMIT_ENDPOINT)
            .json(&payload) // 
            .build()?;

        // Execute through middleware
        let res = client.execute(req).await?;
        tracing::info!("Geosubmit response status: {}", res.status());

        Ok(())
    }
}
