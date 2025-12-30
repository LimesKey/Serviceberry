use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::collections::HashMap;
use std::net::IpAddr;

use crate::config::{HTTP_SERVER_PORT, HTTPS_SERVER_PORT, MDNS_SERVICE_TYPE};

// this variable naming is horrible, hostname::get is a "hostname", but to mDNS it's a "instance/device name".
// And for get_current_username(), it's a "username" but to mDNS it's part of the "hostname".
// Sorry for the confusion!

/// Register the mDNS services for both HTTP and HTTPS
pub fn register_mdns_service(
    device_name: &str, // computer/device name, e.g: "My-MacBook"
    lan_ip: IpAddr,
    version: &str,
    username: &str, // operating system username, e.g: "john"
) -> Result<ServiceDaemon, Box<dyn std::error::Error>> {
    let hostname = format!(
        "{}-{}.local.",
        MDNS_SERVICE_TYPE.to_lowercase(),
        username.to_lowercase()
    );
    let instance_name = format!("{} {}", MDNS_SERVICE_TYPE, device_name);

    let properties_http = HashMap::from([
        ("version".into(), version.into()),
        ("paths".into(), "/status, /request".into()),
    ]);

    let mdns = ServiceDaemon::new()?;

    // HTTP service
    let http_service_type = format!("_{}-http._tcp.local.", MDNS_SERVICE_TYPE.to_lowercase());
    let http_info = ServiceInfo::new(
        &http_service_type, // Service type for discovery
        &instance_name,     // Human-readable instance name
        &hostname,          // DNS name clients connect to
        lan_ip.to_string(),
        HTTP_SERVER_PORT,
        Some(properties_http.clone()),
    )?;
    mdns.register(http_info)?;
    tracing::info!(
        "HTTP mDNS service '{}' published at {}:{}",
        device_name,
        hostname.trim_end_matches('.'),
        HTTP_SERVER_PORT
    );

    let properties_https = HashMap::from([
        ("version".into(), version.into()),
        ("paths".into(), "/submit, /status, /request".into()),
    ]);

    // HTTPS service
    let https_service_type = format!("_{}-https._tcp.local.", MDNS_SERVICE_TYPE.to_lowercase());
    let https_info = ServiceInfo::new(
        &https_service_type, // Service type for discovery
        &instance_name,      // Human-readable instance name
        &hostname,           // DNS name clients connect to
        lan_ip.to_string(),
        HTTPS_SERVER_PORT,
        Some(properties_https),
    )?;
    mdns.register(https_info)?;
    tracing::info!(
        "HTTPS mDNS service '{}' published at {}:{}",
        device_name,
        hostname.trim_end_matches('.'),
        HTTPS_SERVER_PORT
    );

    Ok(mdns)
}
