use btleplug::api::BDAddr as mac_address;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time;

use crate::SCAN_DURATION_SECS;

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct BleDevice {
    #[serde(rename = "macAddress")]
    pub mac_address: mac_address,
    #[serde(rename = "signalStrength", skip_serializing_if = "Option::is_none")]
    pub rssi: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

pub async fn fetch_ble_devices() -> Vec<BleDevice> {
    let mut devices = vec![];

    let manager = match Manager::new().await {
        Ok(m) => m,
        Err(e) => {
            println!("[BLE] Manager error: {:?}", e);
            return devices;
        }
    };

    let adapters = manager.adapters().await.unwrap_or_default();
    let adapter = match adapters.into_iter().next() {
        // check to see if there's at least one bluetooth adapter/card
        Some(a) => a,
        None => {
            println!("[BLE] No adapters found");
            return devices;
        }
    };

    println!("[BLE] Starting BLE scan...");

    if let Err(e) = adapter.start_scan(ScanFilter::default()).await {
        println!("[BLE] Scan failed: {:?}", e);
        return devices;
    }

    time::sleep(Duration::from_secs(SCAN_DURATION_SECS)).await;

    let scan_results = adapter.peripherals().await.unwrap_or_default();

    for broadcaster in scan_results {
        if let Ok(Some(props)) = broadcaster.properties().await {
            let device = BleDevice {
                mac_address: broadcaster.address(),
                rssi: props.rssi,
                name: props.local_name.filter(|n| !n.is_empty()),
            };

            devices.push(device);
        }
    }

    println!("[BLE] Total devices: {}", devices.len());
    devices
}
