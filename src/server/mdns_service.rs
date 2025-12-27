//! mDNS service registration

use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::collections::HashMap;
use std::net::IpAddr;

use crate::config::{MDNS_SERVICE_TYPE, HTTP_SERVER_PORT};

/// Register the mDNS service
pub fn register_mdns_service(
    hostname: &str,
    lan_ip: IpAddr,
    version: &str,
    cert_fingerprint: &[u8; 32],
) -> Result<ServiceDaemon, Box<dyn std::error::Error>> {
    let service_type = format!("_{:?}._tcp.local.", MDNS_SERVICE_TYPE.to_lowercase());

    let properties = HashMap::from([
        ("version".into(), version.into()),
        ("paths".into(), "/submit, /status, /request".into()),
        ("cert_fingerprint".into(), hex::encode(cert_fingerprint).into()),
    ]);

    let service_info: ServiceInfo = ServiceInfo::new(
        &service_type, // Service you're running, ServiceBerry in this case
        hostname, // pretty, human readable name for the device you're using
        "serviceberry.local.", // actual mDNS url name you're broadcasting as / second level domain
        lan_ip.to_string(),
        HTTP_SERVER_PORT,
        Some(properties),
    )?;

    let mdns = ServiceDaemon::new()?;
    mdns.register(service_info)?;

   let mdns_name = format!("{}.local.", hostname);
    println!(
        "mDNS service published as '{}' at {}:{}",
        mdns_name, lan_ip, HTTP_SERVER_PORT
    );

    Ok(mdns)
}
