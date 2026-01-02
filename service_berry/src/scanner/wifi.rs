use core::panic;
use std::time::Duration;

use btleplug::api::BDAddr as mac_address;
use neli_wifi::{Interface, Socket};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::config::{DWELL_TIME, SCAN_DURATION_SECS};

// oh my gosh I wrote all this code before discovering:
// "Do NOT screenscrape this tool, we don't consider its output stable."

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Cipher {
    CCMP,
    TKIP,
    GCMP,
    Unknown(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AuthSuite {
    PSK,
    SAE,
    EAP,
    Unknown(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WPAVersion {
    WPA,
    WPA2,
    WPA3,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WifiSecurity {
    pub version: WPAVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_cipher: Option<Cipher>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pairwise_ciphers: Option<Vec<Cipher>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_suites: Option<Vec<AuthSuite>>,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct WifiBssid {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssid: Option<String>,
    #[serde(rename = "macAddress")]
    pub bssid: mac_address, // a mac adddress for a specific SSID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<u64>, // in milliseconds since last seen
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
    pub frequency: u16, // in MHz
    #[serde(rename = "radioType")]
    pub phy: PhyType, // physcial layer type, usually correlated with wifi versioning
    #[serde(rename = "signalStrength")]
    pub rssi: i32, // Signal Strength, in dBm
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<WifiSecurity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth: Option<u16>, // in MHz
}

#[derive(Serialize, Debug, Clone, Deserialize, PartialEq)]
pub enum PhyType {
    #[serde(rename = "802.11bn")]
    Uhr, // Extremely High Throughput (EHT) / Ultra High Rate
    #[serde(rename = "802.11be")]
    Eht, // Extremely High Throughput
    #[serde(rename = "802.11ax")]
    He, // High Efficiency
    #[serde(rename = "802.11ac")]
    Vht, // Very High Throughput
    #[serde(rename = "802.11n")]
    Ht, // High Throughput
    #[serde(rename = "802.11a/b/g")]
    Legacy, // Anything else (pre-802.11n)
}

// Hidden SSIDs: empty, spaces, or only \xNN escapes
static RE_HIDDEN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?:\\x[0-9A-Fa-f]{2}| )*$").unwrap());

// Fully invalid: only \xNN escapes (no spaces)
static RE_INVALID: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?:\\x[0-9A-Fa-f]{2})+$").unwrap());

// Detects any \xNN escape (used for partial-invalid detection)
static RE_ESCAPE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\x[0-9A-Fa-f]{2}").unwrap());

// Valid UTF-8: at least one printable, no escapes
static RE_VALID_UTF8: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^\x00-\x1F\x7F]").unwrap());

impl WifiBssid {
    #[instrument(skip(self), fields(raw_ssid = raw_ssid))]
    fn parse_ssid(&mut self, raw_ssid: &str) {
        let ssid = raw_ssid.trim();
        debug!(trimmed = ssid, "Parsing SSID");

        // hidden SSID
        if RE_HIDDEN.is_match(ssid) {
            debug!("Skipping hidden SSID");
            self.ssid = None;
            return;
        }

        // fully invalid, pure escapes
        if RE_INVALID.is_match(ssid) {
            warn!(ssid, "Skipping invalid SSID");
            self.ssid = None;
            return;
        }

        if RE_ESCAPE.is_match(ssid) {
            let cleaned = RE_ESCAPE.replace_all(ssid, "").trim().to_string();
            if cleaned.is_empty() {
                error!(ssid, "SSID became empty after cleaning");
                panic!("[WiFi] Error: SSID became empty after cleaning: {}", ssid);
            }
            debug!(cleaned, "Cleaning escaped SSID");
            self.ssid = Some(cleaned);
            return;
        }

        // no escapes,must be valid UTF-8
        if RE_VALID_UTF8.is_match(ssid) {
            trace!("SSID validated as UTF-8 without escapes");
            self.ssid = Some(ssid.to_string());
            return;
        }

        error!(ssid, "SSID did not match any known patterns");
        panic!("[WiFi] SSID did not match any known patterns: {}", ssid);
    }

    fn parse_security(&mut self, line: &str) {
        let has_sae = line.contains("SAE");
        let has_psk = line.contains("PSK");
        let has_eap = line.contains("802.1X") || line.contains("EAP");
        let has_rsn = line.contains("RSN");
        let has_wpa = line.contains("WPA:");

        if has_sae {
            let mut auth = vec![AuthSuite::SAE];
            if has_psk {
                auth.push(AuthSuite::PSK);
            }
            self.security = Some(WifiSecurity {
                version: WPAVersion::WPA3,
                group_cipher: Some(Cipher::CCMP),
                pairwise_ciphers: Some(vec![Cipher::CCMP]),
                auth_suites: Some(auth),
            });
            return;
        }

        // RSN (WPA2) without SAE
        if has_rsn || has_psk {
            let mut auth = Vec::new();
            if has_psk {
                auth.push(AuthSuite::PSK);
            }
            if has_eap {
                auth.push(AuthSuite::EAP);
            }
            self.security = Some(WifiSecurity {
                version: WPAVersion::WPA2,
                group_cipher: Some(Cipher::CCMP),
                pairwise_ciphers: Some(vec![Cipher::CCMP]),
                auth_suites: if auth.is_empty() { None } else { Some(auth) },
            });
            return;
        }

        // Legacy WPA (pre-RSN)
        if has_wpa {
            self.security = Some(WifiSecurity {
                version: WPAVersion::WPA,
                group_cipher: Some(Cipher::TKIP),
                pairwise_ciphers: Some(vec![Cipher::TKIP]),
                auth_suites: Some(vec![AuthSuite::PSK]),
            });
            return;
        }

        // WEP / privacy flag only
        if line.contains("capability:") && line.contains("Privacy") && self.security.is_none() {
            self.security = Some(WifiSecurity {
                version: WPAVersion::WPA, // placeholder
                group_cipher: None,
                pairwise_ciphers: None,
                auth_suites: Some(vec![AuthSuite::Unknown("WEP".into())]),
            });
            return;
        }
    }

    fn update_bandwidth_from_line(&mut self, line: &str) {
        // Try to parse bandwidth from HT/VHT/EHT lines
        if line.contains("HT:") && line.contains("max bandwidth") {
            if let Some(cap) = line.split("max bandwidth:").nth(1) {
                if let Some(bw_str) = cap.split_whitespace().next() {
                    if let Ok(bw) = bw_str.trim_end_matches("MHz").parse::<u16>() {
                        self.bandwidth = Some(bw);
                        return;
                    }
                }
            }
        }

        if line.contains("VHT Capabilities") {
            // VHT typically 80 or 160 MHz, try to parse channel width if present
            if line.contains("channel width 160 MHz") {
                self.bandwidth = Some(160);
            } else {
                self.bandwidth = Some(80);
            }
            return;
        }

        if line.contains("EHT Capabilities") {
            // EHT can go up to 320 MHz
            if line.contains("320 MHz") {
                self.bandwidth = Some(320);
            } else if line.contains("160 MHz") {
                self.bandwidth = Some(160);
            } else {
                self.bandwidth = Some(80);
            }
            return;
        }

        if line.contains("UHR capabilities") {
            self.bandwidth = Some(320);
            return;
        }
    }

    fn set_bandwidth_from_phy(&mut self) {
        // fallback if no bandwidth line found
        if self.bandwidth.is_none() {
            self.bandwidth = match self.phy {
                PhyType::Uhr => Some(320),
                PhyType::Eht => Some(160),
                PhyType::He => Some(160),
                PhyType::Vht => Some(80),
                PhyType::Ht => Some(40),
                PhyType::Legacy => Some(20),
            };
        }
    }
}

#[instrument]
#[cfg(target_os = "linux")]
pub async fn fetch_wifi_stats() -> Vec<WifiBssid> {
    info!("[WIFI] Running WiFi scan");
    let iw_trigger = tokio::process::Command::new("iw")
        .args([
            "dev",
            "wlan0",
            "scan",
            "trigger",
            "flush",
            "duration",
            &DWELL_TIME.to_string(),
            "duration-mandatory",
        ])
        .output()
        .await
        .inspect_err(|e| error!(error = %e, "Failed to trigger scan"))
        .expect("[WiFi] Failed to trigger scan - Is IW installed?"); // Wait 10 seconds for scan to complete 

    let stderr = String::from_utf8_lossy(&iw_trigger.stderr);
    if !stderr.is_empty() {
        warn!(stderr = %stderr, "IW scan trigger stderr");
        panic!(
            "[WiFi] iw scan trigger error, did you enable sudo? error: {}",
            stderr
        );
    }

    trace!(
        duration_secs = SCAN_DURATION_SECS,
        "Waiting for scan to complete"
    );
    tokio::time::sleep(Duration::from_secs(SCAN_DURATION_SECS)).await; // Dump the scan results 
    let output = tokio::process::Command::new("sudo")
        .args(["iw", "dev", "wlan0", "scan", "dump"])
        .output()
        .await
        .inspect_err(|e| error!(error = %e, "Failed to dump scan results"))
        .expect("[WiFi] Failed to dump scan results");
    let stdout = String::from_utf8_lossy(&output.stdout);

    let re_bssid = Regex::new(r"^BSS ([0-9a-f:]{17})").unwrap(); // match for access point mac address
    let re_ssid = Regex::new(r"^\s*SSID:(.*)$").unwrap();
    let re_freq = Regex::new(r"^\s*freq: (\d+)").unwrap();
    let re_channel = Regex::new(r"^\s*\* primary channel: (\d+)").unwrap();
    let re_signal = Regex::new(r"signal:\s*([-]?\d+(?:\.\d+)?) dBm").unwrap(); // in dBm
    let re_last_seen = Regex::new(r"^\s*last seen: (\d+)\s*ms").unwrap(); // in milliseconds

    let re_uhr_caps = Regex::new(r"^\s*UHR capabilities:").unwrap(); // Ultra High Rate Wifi 8 802.11bn
    let re_eht_caps = Regex::new(r"^\s*EHT capabilities:").unwrap(); // Extremely High Throughput Wifi 7 802.11be
    let re_he_caps = Regex::new(r"^\s*HE capabilities:").unwrap(); // High Efficiency Wifi 6 802.11ax
    let re_vht_caps = Regex::new(r"^\s*VHT capabilities:").unwrap(); // Very High Throughput Wifi 5 802.11ac
    let re_ht_caps = Regex::new(r"^\s*HT capabilities:").unwrap(); // High Throughput Wifi 4 802.11n

    let mut bssid_records = Vec::new();
    let mut current_bssid: Option<WifiBssid> = None;

    for line in stdout.lines() {
        trace!(line, "Parsing scan line");
        if let Some(caps) = re_bssid.captures(line) {
            // if new AP is found
            if let Some(mut ap) = current_bssid.take() {
                // check if there was a AP being built
                if ap.phy == PhyType::Legacy {
                    warn!(bssid = %ap.bssid, "PHY type remained Legacy - could not detect capabilities");
                }
                ap.set_bandwidth_from_phy(); // fallback if no bandwidth info was found
                bssid_records.push(ap); // if so, push it to the vec
            }

            current_bssid = Some(WifiBssid {
                ssid: None,
                bssid: caps[1].parse().unwrap_or_default(),
                age: None,
                channel: None,
                frequency: 0,
                phy: PhyType::Legacy,
                rssi: 0,
                security: None,
                bandwidth: None,
            });
        } else if let Some(bssid) = current_bssid.as_mut() {
            // SSID
            if let Some(caps) = re_ssid.captures(line) {
                WifiBssid::parse_ssid(bssid, &caps[1]);
                trace!(bssid = %bssid.bssid, "Parsed SSID");
                continue;
            }

            // Frequency
            if let Some(caps) = re_freq.captures(line) {
                bssid.frequency = caps[1].parse().unwrap_or(0);
                trace!(bssid = %bssid.bssid, frequency = bssid.frequency, "Parsed frequency");
                continue;
            }

            // Channel
            if let Some(caps) = re_channel.captures(line) {
                bssid.channel = caps[1].parse().ok();
                trace!(bssid = %bssid.bssid, channel = ?bssid.channel, "Parsed channel");
                continue;
            }

            // Signal strength
            if let Some(caps) = re_signal.captures(line) {
                bssid.rssi = caps[1].parse::<f64>().unwrap_or(0.0) as i32;
                trace!(bssid = %bssid.bssid, rssi = bssid.rssi, "Parsed signal strength");
                continue;
            }

            // Last seen age
            if let Some(caps) = re_last_seen.captures(line) {
                let age_ms = caps[1].parse::<f64>().unwrap_or(0.0) * 1000.0;
                bssid.age = Some(age_ms as u64);
                trace!(bssid = %bssid.bssid, age = ?bssid.age, "Parsed last seen age");
                continue;
            }

            // Security detection
            bssid.parse_security(line);

            // PHY type detection
            if re_uhr_caps.is_match(line) {
                bssid.phy = PhyType::Uhr;
                bssid.update_bandwidth_from_line(line);
                trace!(bssid = %bssid.bssid, phy = ?bssid.phy, bandwidth = ?bssid.bandwidth, "Updated PHY type to Uhr");
            } else if re_eht_caps.is_match(line) {
                bssid.phy = PhyType::Eht;
                bssid.update_bandwidth_from_line(line);
                trace!(bssid = %bssid.bssid, phy = ?bssid.phy, bandwidth = ?bssid.bandwidth, "Updated PHY type to Eht");
            } else if re_he_caps.is_match(line) {
                bssid.phy = PhyType::He;
                bssid.update_bandwidth_from_line(line);
                trace!(bssid = %bssid.bssid, phy = ?bssid.phy, bandwidth = ?bssid.bandwidth, "Updated PHY type to He");
            } else if re_vht_caps.is_match(line) {
                bssid.phy = PhyType::Vht;
                bssid.update_bandwidth_from_line(line);
                trace!(bssid = %bssid.bssid, phy = ?bssid.phy, bandwidth = ?bssid.bandwidth, "Updated PHY type to Vht");
            } else if re_ht_caps.is_match(line) {
                bssid.phy = PhyType::Ht;
                bssid.update_bandwidth_from_line(line);
                trace!(bssid = %bssid.bssid, phy = ?bssid.phy, bandwidth = ?bssid.bandwidth, "Updated PHY type to Ht");
            } else {
                // For any line not matching capabilities, try to parse bandwidth directly
                bssid.update_bandwidth_from_line(line);
            }

            // Fallback to PHY defaults if no bandwidth was found
            if bssid.bandwidth.is_none() {
                bssid.set_bandwidth_from_phy();
            }
        }
    }

    if let Some(mut bssid) = current_bssid {
        if bssid.security.is_none() {
            bssid.security = Some(WifiSecurity {
                version: WPAVersion::WPA, // placeholder for open
                group_cipher: None,
                pairwise_ciphers: None,
                auth_suites: None,
            });
        }
        bssid.set_bandwidth_from_phy();
        bssid_records.push(bssid);
    }

    info!("Finished scanning found {} BSSIDs", bssid_records.len());
    println!("[WIFI] Found {} BSSIDs", bssid_records.len());
    bssid_records
}

#[cfg(target_os = "linux")]
pub fn fetch_wifi_interfaces() -> Result<Vec<Interface>, Box<dyn Error>> {
    let mut socket = Socket::connect()?;
    let mut interfaces = Vec::new();

    for interface in socket.get_interfaces_info()? {
        if let Some(index) = interface.index {
            debug!("interface {} info: {:?}", index, interface);
        }

        interfaces.push(interface);
    }

    Ok(interfaces)
}
