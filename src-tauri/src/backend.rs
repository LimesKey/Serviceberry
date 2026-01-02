use serde_json::Value;

#[tauri::command]
pub async fn list_wifi() -> Result<Value, String> {
    let res = service_berry::scanner::wifi::fetch_wifi_stats().await;
    serde_json::to_value(&res).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_adapters() -> Result<Value, String> {
    let interfaces =
        service_berry::scanner::wifi::fetch_wifi_interfaces().map_err(|e| e.to_string())?;
    let names: Vec<String> = interfaces
        .into_iter()
        .filter_map(|iface| iface.name.map(|n| String::from_utf8_lossy(&n).to_string()))
        .collect();
    serde_json::to_value(names).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_bt() -> Result<Value, String> {
    // fetch devices from the Rust scanner, then normalize JSON keys so the frontend
    // receives consistent fields: `address`, `rssi`, `name`, `uuids` (if present).
    let res = service_berry::scanner::bluetooth::fetch_ble_devices().await;
    let mapped: Result<Vec<Value>, String> = res
        .into_iter()
        .map(|dev| {
            let v = serde_json::to_value(dev).map_err(|e| e.to_string())?;
            if let Value::Object(mut m) = v {
                // rename macAddress -> address
                if let Some(mac) = m.remove("macAddress") {
                    m.insert("address".to_string(), mac);
                }
                // rename signalStrength -> rssi
                if let Some(sig) = m.remove("signalStrength") {
                    m.insert("rssi".to_string(), sig);
                }
                // keep name as-is; ensure uuids present as array if missing
                if !m.contains_key("uuids") {
                    m.insert("uuids".to_string(), Value::Array(vec![]));
                }
                Ok(Value::Object(m))
            } else {
                Ok(v)
            }
        })
        .collect();
    serde_json::to_value(mapped.map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn submit_geo(position: Value, cell_towers: Option<Value>) -> Result<String, String> {
    let items = service_berry::geosubmit::assemble_geo_payload(position, cell_towers)
        .await
        .map_err(|e| e.to_string())?;

    service_berry::geosubmit::submit_geo_payload(items)
        .await
        .map_err(|e| e.to_string())?;

    Ok("ok".into())
}

pub async fn run_backend() -> Result<(), String> {
    let cfg = service_berry::config::Config {
        wifi_adapter: "wlan0".to_string(),
        http_server_port: service_berry::config::HTTP_SERVER_PORT,
        https_server_port: service_berry::config::HTTPS_SERVER_PORT,
        scan_duration_secs: service_berry::config::SCAN_DURATION_SECS,
        dwell_time: service_berry::config::DWELL_TIME,
        geosubmit_endpoint: service_berry::config::GEOSUBMIT_ENDPOINT.to_string(),
        user_agent: service_berry::config::APP_USER_AGENT.to_string(),
        directory: service_berry::config::Config::get_config_dir(),
    };

    service_berry::run(cfg).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_backend_command() -> Result<String, String> {
    run_backend().await.map(|_| "ok".into())
}
