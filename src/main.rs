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

    // // Start BLE peripheral in background
    // let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // tokio::spawn(async move {
    //     peripheral::ble_peripheral(rx).await;
    // });

    // tx.send("Hello iOS".to_string()).ok();

    // Start HTTP server
    server::start_tls(identity).await?;

    Ok(())
}
