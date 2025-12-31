use core::panic;
use std::time::Duration;

use btleplug::api::BDAddr as mac_address;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::config::{SCAN_DURATION_SECS, DWELL_TIME};

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

#[derive(Serialize, Debug, Clone, Deserialize, PartialEq)]
pub enum PhyType {
    #[serde(rename = "802.11bn")]
    Uhr,    // Extremely High Throughput (EHT) / Ultra High Rate
    #[serde(rename = "802.11be")]
    Eht,    // Extremely High Throughput
    #[serde(rename = "802.11ax")]
    He,     // High Efficiency
    #[serde(rename = "802.11ac")]
    Vht,    // Very High Throughput
    #[serde(rename = "802.11n")]
    Ht,     // High Throughput
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

        // Contains escapes > partial invalid > clean it
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
}

#[instrument]
pub async fn fetch_wifi_stats() -> Vec<WifiBssid> {
    info!("Running WiFi scan");
    let _ = tokio::process::Command::new("sudo")
        .args(["iw", "dev", "wlan0", "scan", "trigger", "duration", &DWELL_TIME.to_string()])
        .output()
        .await
        .inspect_err(|e| error!(error = %e, "Failed to trigger scan"))
        .expect("[WiFi] Failed to trigger scan - Is IW installed?"); // Wait 10 seconds for scan to complete 

    trace!(duration_secs = SCAN_DURATION_SECS, "Waiting for scan to complete");
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
            if let Some(ap) = current_bssid.take() {
                // check if there was a AP being built
                if ap.phy == PhyType::Legacy {
                    warn!(bssid = %ap.bssid, "PHY type remained Legacy - could not detect capabilities");
                }
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

            // PHY type detection
            if re_uhr_caps.is_match(line) {
                bssid.phy = PhyType::Uhr;
                trace!(bssid = %bssid.bssid, phy = ?bssid.phy, "Updated PHY type to Uhr");
            } else if re_eht_caps.is_match(line) {
                bssid.phy = PhyType::Eht;
                trace!(bssid = %bssid.bssid, phy = ?bssid.phy, "Updated PHY type to Eht");
            } else if re_he_caps.is_match(line) {
                bssid.phy = PhyType::He;
                trace!(bssid = %bssid.bssid, phy = ?bssid.phy, "Updated PHY type to He");
            } else if re_vht_caps.is_match(line) {
                bssid.phy = PhyType::Vht;
                trace!(bssid = %bssid.bssid, phy = ?bssid.phy, "Updated PHY type to Vht");
            } else if re_ht_caps.is_match(line) {
                bssid.phy = PhyType::Ht;
                trace!(bssid = %bssid.bssid, phy = ?bssid.phy, "Updated PHY type to Ht");
            }
        }
    }

        
    if let Some(bssid) = current_bssid {
        bssid_records.push(bssid);
    }

    info!(total = bssid_records.len(), "Finished scanning");
    bssid_records
}
