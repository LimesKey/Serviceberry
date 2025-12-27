//! ServiceBerry - Geolocation service via WiFi & Bluetooth scanning
//!
//! A service that scans nearby WiFi and Bluetooth devices and submits
//! location data to the Ichnaea geolocation service.

use local_ip_address::local_ip;
use service_berry::{config, peripheral, server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get system info
    let hostname = hostname::get()
        .unwrap_or_else(|_| config::DEFAULT_HOSTNAME.into())
        .to_string_lossy()
        .to_string();
    let version = env!("CARGO_PKG_VERSION");
    let lan_ip = local_ip().expect("Could not get local IP address");

    println!("Starting ServiceBerry v{} on {}", version, hostname);

    // Generate TLS certificates
    let config_directory = config::config_dir();
    let identity = config::load_identity(hostname.clone(), config_directory)?;

    // Register mDNS service
    let _mdns = server::mdns_service::register_mdns_service(
        &hostname,
        lan_ip,
        version,
        &identity.certs_hash,
    )
    .map_err(|e| format!("Failed to register mDNS: {}", e))?;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<server::handlers::PartialPayload>();

    // Start the BLE peripheral
    tokio::spawn(async move {
        peripheral::ble_peripheral(tx).await;
    });

    // Start the Worker
    tokio::spawn(async move {
        while let Some(payload) = rx.recv().await {
            tracing::info!("Worker received payload from BLE: {:?}", payload);
            // This is where you call your submission logic
            if let Err(e) = server::handlers::process_submit(payload).await {
                tracing::error!("Failed to process BLE submission: {:?}", e);
            }
        }
    });

    // Start HTTP server
    server::start_tls(identity).await?;

    Ok(())
}
