use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::collections::HashMap;
use std::net::IpAddr;

use crate::config::{HTTP_SERVER_PORT, MDNS_SERVICE_TYPE};

/// Register the mDNS service
pub fn register_mdns_service(
    instance_name: &str,
    lan_ip: IpAddr,
    version: &str,
    cert_fingerprint: &[u8; 32],
    username: &str,
) -> Result<ServiceDaemon, Box<dyn std::error::Error>> {
    let service_type = format!("_{}._tcp.local.", MDNS_SERVICE_TYPE.to_lowercase());

    let properties = HashMap::from([
        ("version".into(), version.into()),
        ("paths".into(), "/submit, /status, /request".into()),
        (
            "cert_fingerprint".into(),
            hex::encode(cert_fingerprint).into(),
        ),
    ]);

    let hostname = format!("serviceberry-{}.local.", username.to_lowercase());

    tracing::info!("Registering mDNS service '{}'", service_type);
    tracing::debug!("Service type: {}", service_type);
    tracing::debug!("Hostname: {}", hostname);
    tracing::debug!("LAN IP: {}", lan_ip);
    tracing::debug!("Port: {}", HTTP_SERVER_PORT);
    tracing::debug!("TXT Properties: {:?}", properties);

    let service_info: ServiceInfo = ServiceInfo::new(
        &service_type, // Service type for discovery
        instance_name, // Human-readable instance name
        &hostname,     // DNS name clients connect to
        lan_ip.to_string(),
        HTTP_SERVER_PORT,
        Some(properties),
    )?;

    let mdns = ServiceDaemon::new()?;
    mdns.register(service_info)?;

    tracing::info!(
        "mDNS service '{}' successfully published at {}:{}",
        instance_name,
        hostname.trim_end_matches('.'),
        HTTP_SERVER_PORT
    );

    Ok(mdns)
}
