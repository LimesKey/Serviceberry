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
    let instance_name = device_name;

    let properties = HashMap::from([
        ("version".into(), version.into()),
        ("http".into(), HTTP_SERVER_PORT.to_string()),
        ("https".into(), HTTPS_SERVER_PORT.to_string()),
        ("paths".into(), "/submit,/status,/request".into()),
    ]);

    let mdns = ServiceDaemon::new()?;

    let service_type = format!("_{}._tcp.local.", MDNS_SERVICE_TYPE.to_lowercase());
    let service_info = ServiceInfo::new(
        &service_type, // Service type for discovery
        instance_name, // Human-readable instance name
        &hostname,     // DNS name clients connect to
        lan_ip.to_string(),
        HTTPS_SERVER_PORT,
        Some(properties),
    )?;
    mdns.register(service_info)?;
    tracing::info!(
        "mDNS service '{}' published at {}:{}",
        device_name,
        hostname.trim_end_matches('.'),
        HTTPS_SERVER_PORT,
    );

    Ok(mdns)
}