use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WifiBssid {
    #[serde(rename = "macAddress")]
    pub bssid: String,
    #[serde(rename = "signalStrength")]
    pub rssi: i32,
}

#[allow(nonstandard_style)]
pub struct GeoProviderRequest {
    pub wifiAccessPoints: Vec<WifiBssid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeoProviderResponse {
    pub location: Location,
    pub accuracy: f64,
}