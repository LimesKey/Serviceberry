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
    let instance_name = hostname::get() // computer name
        .unwrap_or_else(|_| config::DEFAULT_HOSTNAME.into())
        .to_string_lossy()
        .to_string();
    let username = get_current_username() // system username
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let version = env!("CARGO_PKG_VERSION");
    let lan_ip = local_ip().expect("Could not get local IP address");

    println!("Starting ServiceBerry v{} on {}", version, instance_name);

    // Generate TLS certificates
    let config_directory = config::config_dir();
    let identity = config::load_identity(instance_name.clone(), config_directory)?;

    // Register mDNS service
    let _mdns = server::mdns_service::register_mdns_service(
        &instance_name,
        lan_ip,
        version,
        &identity.certs_hash,
        &username,
    )
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
                if let Err(e) = server::handlers::process_submit(payload).await {
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
