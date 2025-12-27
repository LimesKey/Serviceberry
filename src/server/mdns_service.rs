//! mDNS service registration

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
) -> Result<ServiceDaemon, Box<dyn std::error::Error>> {
    let service_type = format!("_{:?}._tcp.local.", MDNS_SERVICE_TYPE.to_lowercase());

    let properties = HashMap::from([
        ("version".into(), version.into()),
        ("paths".into(), "/submit, /status, /request".into()),
        (
            "cert_fingerprint".into(),
            hex::encode(cert_fingerprint).into(),
        ),
    ]);

    let url = String::from("serviceberry.local.");

    let service_info: ServiceInfo = ServiceInfo::new(
        &service_type, // Service you're running, ServiceBerry in this case
        instance_name, // pretty, human readable name for the device you're using
        &url,          // actual mDNS url name you're broadcasting as / second level domain
        lan_ip.to_string(),
        HTTP_SERVER_PORT,
        Some(properties),
    )?;

    let mdns = ServiceDaemon::new()?;
    mdns.register(service_info)?;

    println!(
        "mDNS service published as '{}' at {}:{}",
        url, lan_ip, HTTP_SERVER_PORT
    );

    Ok(mdns)
}
