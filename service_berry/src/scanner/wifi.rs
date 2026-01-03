//! Wi-Fi scanning module.
pub mod nl80211;
pub mod types;

use std::error::Error;
use colored::Colorize;
#[cfg(target_os = "linux")]
use neli_wifi::Interface;
use types::{PhyType, WifiBssid, WpaVersion};

/// Scan for WiFi networks
pub async fn scan(ifindex: u32) -> Result<Vec<WifiBssid>, Box<dyn Error + Send + Sync>> {
    nl80211::scan(ifindex).await
}

/// Display scanned networks to stdout
pub fn display_networks(networks: &[WifiBssid]) {
    if networks.is_empty() {
        return println!("No networks found.");
    }

    let mut sorted = networks.to_vec();
    sorted.sort_by(|a, b| b.rssi.cmp(&a.rssi).then_with(|| a.ssid.cmp(&b.ssid)));

    for net in sorted {
        println!("───────────────────────────────────");
        let ssid = net.ssid.as_deref().unwrap_or("<hidden>");
        let colored = match net.rssi {
            -50.. => ssid.green().bold(),
            -65.. => ssid.yellow().bold(),
            _ => ssid.red().bold(),
        };
        println!("SSID: {colored}");

        let b = net.bssid.into_inner();
        println!(
            "BSSID: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            b[0], b[1], b[2], b[3], b[4], b[5]
        );

        let ci = &net.channel_info;
        let band = match ci.frequency_mhz {
            2400..=2499 => "2.4G",
            5000..=5899 => "5G",
            5925..=7125 => "6G",
            _ => "?",
        };
        println!(
            "Freq: {} MHz (ch {} / {band})",
            ci.frequency_mhz,
            ci.channel.map_or("?".into(), |c| c.to_string())
        );
        println!("Signal: {} dBm", net.rssi);
        if let Some(w) = ci.bandwidth_mhz {
            println!("Width: {w} MHz");
        }

        let sec = &net.wifi_security;
        println!(
            "Security: {}",
            sec.version.map_or("Open", |v| match v {
                WpaVersion::Wpa3 => "WPA3",
                WpaVersion::Wpa2 => "WPA2",
                WpaVersion::Wpa => "WPA",
            })
        );
        if let Some(ref g) = sec.group_cipher {
            println!("Group: {g}");
        }
        if !sec.pairwise_ciphers.is_empty() {
            println!(
                "Pairwise: {}",
                sec.pairwise_ciphers
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        if !sec.auth_suites.is_empty() {
            println!(
                "Auth: {}",
                sec.auth_suites
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        if let Some(age) = net.age {
            println!("Last seen: {age} ms ago");
        }

        println!(
            "PHY: {}",
            net.phy
                .iter()
                .map(|p| match p {
                    PhyType::Uhr => "UHR",
                    PhyType::Eht => "EHT",
                    PhyType::He => "HE",
                    PhyType::Vht => "VHT",
                    PhyType::Ht => "HT",
                    PhyType::Legacy => "Legacy",
                })
                .collect::<Vec<_>>()
                .join(" ")
        );
        if !net.capabilities.is_empty() {
            println!("Capabilities: {}", net.capabilities.join(" "));
        }
    }
}

#[cfg(target_os = "linux")]
pub fn fetch_wifi_interfaces() -> Result<Vec<Interface>, Box<dyn Error>> {
    use neli_wifi::Socket;

    let mut socket = Socket::connect()?;
    let mut interfaces = Vec::new();

    for interface in socket.get_interfaces_info()? {
        if let Some(index) = interface.index {
            use tracing::debug;

            debug!("interface {} info: {:?}", index, interface);
        }

        interfaces.push(interface);
    }

    Ok(interfaces)
}