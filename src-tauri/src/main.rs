//! ServiceBerry - Geolocation service via WiFi & Bluetooth scanning

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use local_ip_address::local_ip;
use service_berry::{
    config, peripheral,
    server::{self, handlers::PartialPayload},
};
use tokio::sync::mpsc;
use tracing::info;
use users::get_current_username;

fn main() {
    // Logging must be initialized BEFORE Tauri starts
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .setup(|_app| {
            tauri::async_runtime::spawn(async move {
                if let Err(e) = run_backend().await {
                    tracing::error!("Backend failed: {:?}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn run_backend() -> Result<(), Box<dyn std::error::Error>> {
    // System info
    let hostname = hostname::get()
        .unwrap_or_else(|_| "unknown-device-hostname".into())
        .to_string_lossy()
        .to_string();

    let username = get_current_username()
        .expect("Cannot retrieve operating system username!")
        .to_string_lossy()
        .to_string();

    let version = env!("CARGO_PKG_VERSION");
    let lan_ip = local_ip().expect("Could not get local IP address");

    info!("Local IP address: {}", lan_ip);

    let mdns_hostname = format!(
        "{}-{}.local",
        config::MDNS_SERVICE_TYPE.to_lowercase(),
        username.to_lowercase()
    );

    info!("Starting ServiceBerry v{} on {}", version, hostname);

    // TLS identity
    let config_directory = config::config_dir();
    let identity = config::load_identity(&mdns_hostname, config_directory)?;

    // mDNS
    let _mdns = server::mdns_service::register_mdns_service(&hostname, lan_ip, version, &username)
        .map_err(|e| format!("Failed to register mDNS: {}", e))?;

    let (tx, mut rx) = mpsc::channel::<PartialPayload>(100);

    // BLE peripheral
    tauri::async_runtime::spawn(async move {
        peripheral::ble_peripheral(tx).await;
    });

    // Payload workers
    tauri::async_runtime::spawn(async move {
        while let Some(payload) = rx.recv().await {
            tauri::async_runtime::spawn(async move {
                tracing::debug!("Worker processing payload: {:?}", payload);
                if let Err(e) = server::handlers::submit_payload(payload).await {
                    tracing::error!("Failed to process submission: {:?}", e);
                }
            });
        }
    });

    // HTTP server
    tauri::async_runtime::spawn(async {
        if let Err(e) = server::start_http().await {
            tracing::error!("HTTP server error: {:?}", e);
        }
    });

    // HTTPS blocks
    server::start_https(identity).await?;

    Ok(())
}
