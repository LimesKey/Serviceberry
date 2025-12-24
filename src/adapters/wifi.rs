use core::panic;
use std::time::Duration;

use btleplug::api::BDAddr as mac_address;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::SCAN_DURATION_SECS;

// oh my gosh I wrote all this code before discovering:
// "Do NOT screenscrape this tool, we don't consider its output stable."

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
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub enum PhyType {
    Uhr,
    Eht,
    He,
    Vht,
    Ht,
    Legacy, // anything not matching above
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
    fn parse_ssid(&mut self, raw_ssid: &str) {
        let ssid = raw_ssid.trim();

        // hidden SSID
        if RE_HIDDEN.is_match(ssid) {
            println!("[WiFi] Skipping hidden SSID");
            self.ssid = None;
        }

        // fully invalid, pure escapes
        if RE_INVALID.is_match(ssid) {
            println!("[WiFi] Skipping invalid SSID: {}", ssid);
            self.ssid = None;
        }

        // Contains escapes > partial invalid > clean it
        if RE_ESCAPE.is_match(ssid) {
            let cleaned = RE_ESCAPE.replace_all(ssid, "").trim().to_string();
            if cleaned.is_empty() {
                panic!("[WiFi] Error: SSID became empty after cleaning: {}", ssid);
            }
            self.ssid = Some(cleaned);
        }

        // no escapes,must be valid UTF-8
        if RE_VALID_UTF8.is_match(ssid) {
            self.ssid = Some(ssid.to_string());
        }

        panic!("[WiFi] SSID did not match any known patterns: {}", ssid);
    }
}

pub async fn fetch_wifi_stats() -> Vec<WifiBssid> {
    println!("[WiFi] Running scan...");
    let _ = tokio::process::Command::new("sudo")
        .args(["iw", "dev", "wlan0", "scan", "trigger"])
        .output()
        .await
        .expect("[WiFi] Failed to trigger scan - Is IW installed?"); // Wait 10 seconds for scan to complete 

    tokio::time::sleep(Duration::from_secs(SCAN_DURATION_SECS)).await; // Dump the scan results 
    let output = tokio::process::Command::new("sudo")
        .args(["iw", "dev", "wlan0", "scan", "dump"])
        .output()
        .await
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
        if let Some(caps) = re_bssid.captures(line) {
            // if new AP is found
            if let Some(ap) = current_bssid.take() {
                // check if there was a AP being built
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
            });
        } else if let Some(bssid) = current_bssid.as_mut() {
            // SSID
            if let Some(caps) = re_ssid.captures(line) {
                WifiBssid::parse_ssid(bssid, &caps[1]);
                continue;
            }

            // Frequency
            if let Some(caps) = re_freq.captures(line) {
                bssid.frequency = caps[1].parse().unwrap_or(0);
                continue;
            }

            // Channel
            if let Some(caps) = re_channel.captures(line) {
                bssid.channel = caps[1].parse().ok();
                continue;
            }

            // Signal strength
            if let Some(caps) = re_signal.captures(line) {
                bssid.rssi = caps[1].parse::<f64>().unwrap_or(0.0) as i32;
                continue;
            }

            // Last seen age
            if let Some(caps) = re_last_seen.captures(line) {
                let age_ms = caps[1].parse::<f64>().unwrap_or(0.0) * 1000.0;
                bssid.age = Some(age_ms as u64);
                continue;
            }

            // PHY type detection
            bssid.phy = if re_uhr_caps.is_match(line) {
                PhyType::Uhr
            } else if re_eht_caps.is_match(line) {
                PhyType::Eht
            } else if re_he_caps.is_match(line) {
                PhyType::He
            } else if re_vht_caps.is_match(line) {
                PhyType::Vht
            } else if re_ht_caps.is_match(line) {
                PhyType::Ht
            } else {
                PhyType::Legacy
            };
        }
    }

    if let Some(bssid) = current_bssid {
        bssid_records.push(bssid);
    }

    println!(
        "[WiFi] Finished scanning. Total Networks: {}",
        bssid_records.len()
    );
    bssid_records
}
