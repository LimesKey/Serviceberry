use crate::{geosubmit::payload::cache_file_path, items};
use humantime::format_duration;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tracing::info;

// see https://searchfox.org/firefox-main/source/dom/system/NetworkGeolocationProvider.sys.mjs

#[derive(Serialize, Deserialize)]
pub struct WifiBssid {
    #[serde(rename = "macAddress")]
    pub bssid: String,
    #[serde(rename = "signalStrength")]
    pub rssi: i32,
}

#[derive(Serialize, Deserialize)]
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

fn load_last_payload() -> Result<items, Box<dyn std::error::Error>> {
    let path = cache_file_path();
    if !path.exists() {
        return Err("Cache file does not exist".into());
    }
    let s = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&s).unwrap())
}

pub fn geo_provider_response(_wifi_bssids: GeoProviderRequest) -> GeoProviderResponse {
    let items = load_last_payload().unwrap();

    // todo: do something with wifi_bssids later for verification

    let location = Location {
        lat: items.position.latitude,
        lng: items.position.longitude,
    };

    info!(
        "Provided cached location from {}",
        relative_time_from_epoch_ms(items.timestamp)
    );

    GeoProviderResponse {
        location,
        accuracy: items.position.accuracy,
    }
}

pub fn relative_time_from_epoch_ms(epoch_ms: u128) -> String {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX_EPOCH")
        .as_millis() as i128;

    let delta_ms = epoch_ms as i128 - now_ms;
    let past = delta_ms < 0;

    let d = Duration::from_millis(delta_ms.unsigned_abs() as u64);

    if past {
        format!("{} ago", format_duration(d))
    } else {
        format!("in {}", format_duration(d))
    }
}
