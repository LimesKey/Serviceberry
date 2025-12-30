//! ServiceBerry - Geolocation service via WiFi & Bluetooth scanning
//!
//! A service that scans nearby WiFi and Bluetooth devices and submits
//! location data to the Ichnaea geolocation service.

use local_ip_address::local_ip;
use service_berry::{
    config, peripheral,
    server::{self, handlers::PartialPayload},
};
use tokio::sync::mpsc;
use users::get_current_username;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // get system info
    let hostname = hostname::get() // computer/device name, e.g: "My-MacBook"
        .unwrap_or_else(|_| "unknown-device-hostname".into())
        .to_string_lossy()
        .to_string();
    let username = get_current_username() // operating system username, e.g: "john"
        .expect("Cannot retrieve operating system username!")
        .to_string_lossy()
        .to_string();
    let version = env!("CARGO_PKG_VERSION");
    let lan_ip = local_ip().expect("Could not get local IP address");
    let mdns_hostname = format!(
        "{}-{}.local",
        config::MDNS_SERVICE_TYPE.to_lowercase(),
        username.to_lowercase()
    );

    println!("Starting ServiceBerry v{} on {}", version, hostname);

    // Generate TLS certificates
    let config_directory = config::config_dir();
    let identity = config::load_identity(&mdns_hostname, config_directory)?;

    // Register mDNS service
    let _mdns = server::mdns_service::register_mdns_service(&hostname, lan_ip, version, &username)
        .map_err(|e| format!("Failed to register mDNS: {}", e))?;

    let (tx, mut rx) = mpsc::channel::<PartialPayload>(100);

    // Start the BLE peripheral
    tokio::spawn(async move {
        peripheral::ble_peripheral(tx).await;
    });

    tokio::spawn(async move {
        while let Some(payload) = rx.recv().await {
            tokio::spawn(async move {
                tracing::debug!("Worker processing payload: {:?}", payload);
                if let Err(e) = server::handlers::submit_payload(payload).await {
                    tracing::error!("Failed to process submission: {:?}", e);
                }
            });
        }
    });

    // Start HTTP server
    tokio::spawn(async {
        if let Err(e) = server::start_http().await {
            tracing::error!("HTTP server error: {:?}", e);
        }
    });

    server::start_https(identity).await?;
    Ok(())
}
