//! Geosubmit API payload types

use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Clone)]
#[allow(nonstandard_style)]
pub struct items {
    pub timestamp: u128,
    /// Timestamp in milliseconds since Unix epoch
    pub position: Position,
    pub bluetoothBeacons: Vec<crate::scanner::BleDevice>,
    pub wifiAccessPoints: Vec<crate::scanner::WifiBssid>,
    pub CellTowers: Option<Vec<CellTower>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct Position {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    pub altitude: f64,
    pub altitudeAccuracy: f64,
    pub heading: f64,
    pub speed: f64,
    pub source: String,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct CellTower {
    pub radioType: Option<RadioType>, // "gsm", "wcdma", or "lte"
    pub mobileCountryCode: u16,       // MCC
    pub mobileNetworkCode: u16,       // MNC
    pub locationAreaCode: u32,        // LAC (GSM/WCDMA) or TAC (LTE)
    pub cellId: u32,                  // Cell Identity
    pub age: Option<u32>,             // ms since last seen
    pub asu: Option<u8>,              // Arbitrary Strength Unit
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[allow(non_camel_case_types)]
pub enum RadioType {
    #[serde(rename = "gsm")]
    Gsm,
    #[serde(rename = "wcdma")]
    Wcdma,
    #[serde(rename = "lte")]
    Lte,
}

impl CellTower {
    /// Set radio type from string
    pub fn set_radio_type(&mut self, radio: &str) {
        self.radioType = match radio.to_lowercase().as_str() {
            "gsm" => Some(RadioType::Gsm),
            "wcdma" => Some(RadioType::Wcdma),
            "lte" => Some(RadioType::Lte),
            _ => None,
        }
    }
}
