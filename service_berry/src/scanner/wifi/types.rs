//! WiFi scanning types and enums.

use btleplug::api::BDAddr as MacAddress;
use serde::{Deserialize, Serialize};

/// Cipher suite for encryption (mirrors wl-nl80211's Nl80211CipherSuit with serde)
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CipherSuite {
    /// Use group cipher suite
    UseGroup,
    /// WEP-40 (deprecated, insecure)
    Wep40,
    /// TKIP (deprecated, WPA1 era)
    Tkip,
    /// CCMP-128 / AES-CCMP (WPA2/WPA3 standard)
    Ccmp,
    /// WEP-104 (deprecated, insecure)
    Wep104,
    /// AES-CMAC (for management frame protection)
    AesCmac,
    /// GCMP-128 (WPA3)
    Gcmp,
    /// GCMP-256 (WPA3-Enterprise 192-bit)
    Gcmp256,
    /// CCMP-256 (WPA3-Enterprise 192-bit)
    Ccmp256,
    /// BIP-GMAC-128 (management frame protection)
    BipGmac128,
    /// BIP-GMAC-256 (management frame protection)
    BipGmac256,
    /// BIP-CMAC-256 (management frame protection)
    BipCmac256,
    /// SMS4 (Chinese WAPI standard)
    Sms4,
    /// Unknown/other cipher with OUI
    Other(u32),
}

impl std::fmt::Display for CipherSuite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UseGroup => write!(f, "UseGroup"),
            Self::Wep40 => write!(f, "WEP-40"),
            Self::Tkip => write!(f, "TKIP"),
            Self::Ccmp => write!(f, "CCMP"),
            Self::Wep104 => write!(f, "WEP-104"),
            Self::AesCmac => write!(f, "AES-CMAC"),
            Self::Gcmp => write!(f, "GCMP"),
            Self::Gcmp256 => write!(f, "GCMP-256"),
            Self::Ccmp256 => write!(f, "CCMP-256"),
            Self::BipGmac128 => write!(f, "BIP-GMAC-128"),
            Self::BipGmac256 => write!(f, "BIP-GMAC-256"),
            Self::BipCmac256 => write!(f, "BIP-CMAC-256"),
            Self::Sms4 => write!(f, "SMS4"),
            Self::Other(id) => write!(f, "Other(0x{:08x})", id),
        }
    }
}

/// Authentication and Key Management suite
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AkmSuite {
    /// Pre-Shared Key (WPA2-Personal)
    Psk,
    /// Simultaneous Authentication of Equals (WPA3-Personal)
    Sae,
    /// IEEE 802.1X (WPA2-Enterprise)
    Ieee8021x,
    /// PSK with SHA-256 (WPA2-Personal enhanced)
    PskSha256,
    /// 802.1X with SHA-256 (WPA2-Enterprise enhanced)
    Ieee8021xSha256,
    /// Fast Transition with PSK
    FtPsk,
    /// Fast Transition with SAE
    FtSae,
    /// Fast Transition with 802.1X
    FtIeee8021x,
    /// Suite-B (WPA3-Enterprise)
    SuiteB,
    /// Suite-B 192-bit (WPA3-Enterprise 192-bit mode)
    SuiteB192,
    /// Opportunistic Wireless Encryption (OWE) - WPA3 open
    Owe,
    /// FILS with SHA-256
    FilsSha256,
    /// FILS with SHA-384
    FilsSha384,
    /// Unknown/other AKM
    Other,
}

impl std::fmt::Display for AkmSuite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Psk => write!(f, "PSK"),
            Self::Sae => write!(f, "SAE"),
            Self::Ieee8021x => write!(f, "802.1X"),
            Self::PskSha256 => write!(f, "PSK-SHA256"),
            Self::Ieee8021xSha256 => write!(f, "802.1X-SHA256"),
            Self::FtPsk => write!(f, "FT-PSK"),
            Self::FtSae => write!(f, "FT-SAE"),
            Self::FtIeee8021x => write!(f, "FT-802.1X"),
            Self::SuiteB => write!(f, "Suite-B"),
            Self::SuiteB192 => write!(f, "Suite-B-192"),
            Self::Owe => write!(f, "OWE"),
            Self::FilsSha256 => write!(f, "FILS-SHA256"),
            Self::FilsSha384 => write!(f, "FILS-SHA384"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// WPA protocol version
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpaVersion {
    /// WPA (legacy, uses TKIP)
    Wpa,
    /// WPA2 (RSN, uses CCMP)
    Wpa2,
    /// WPA3 (RSN with SAE/OWE)
    Wpa3,
}

impl std::fmt::Display for WpaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wpa => write!(f, "WPA"),
            Self::Wpa2 => write!(f, "WPA2"),
            Self::Wpa3 => write!(f, "WPA3"),
        }
    }
}

/// Security configuration for a network
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Security {
    /// WPA version (None = Open network)
    pub version: Option<WpaVersion>,
    /// Group/broadcast cipher
    pub group_cipher: Option<CipherSuite>,
    /// Pairwise/unicast ciphers (empty for open networks)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub pairwise_ciphers: Vec<CipherSuite>,
    /// Authentication suites (empty for open networks)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub auth_suites: Vec<AkmSuite>,
    /// Management Frame Protection capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfp: Option<MfpCapability>,
}

/// Management Frame Protection capability
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum MfpCapability {
    /// MFP not supported
    None,
    /// MFP capable but optional
    Capable,
    /// MFP required
    Required,
}

/// Physical layer type (WiFi generation)
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhyType {
    /// 802.11bn - Ultra High Rate (WiFi 8, future)
    #[serde(rename = "802.11bn")]
    Uhr,
    /// 802.11be - Extremely High Throughput (WiFi 7)
    #[serde(rename = "802.11be")]
    Eht,
    /// 802.11ax - High Efficiency (WiFi 6/6E)
    #[serde(rename = "802.11ax")]
    He,
    /// 802.11ac - Very High Throughput (WiFi 5)
    #[serde(rename = "802.11ac")]
    Vht,
    /// 802.11n - High Throughput (WiFi 4)
    #[serde(rename = "802.11n")]
    Ht,
    /// 802.11a/b/g - Legacy (pre-WiFi 4)
    #[serde(rename = "802.11a/b/g")]
    Legacy,
}

impl PhyType {
    /// Get the WiFi marketing name (e.g., "WiFi 6")
    pub fn wifi_name(&self) -> &'static str {
        match self {
            Self::Uhr => "WiFi 8",
            Self::Eht => "WiFi 7",
            Self::He => "WiFi 6",
            Self::Vht => "WiFi 5",
            Self::Ht => "WiFi 4",
            Self::Legacy => "Legacy",
        }
    }

    /// Get the IEEE standard name
    pub fn ieee_name(&self) -> &'static str {
        match self {
            Self::Uhr => "802.11bn",
            Self::Eht => "802.11be",
            Self::He => "802.11ax",
            Self::Vht => "802.11ac",
            Self::Ht => "802.11n",
            Self::Legacy => "802.11a/b/g",
        }
    }
}

impl std::fmt::Display for PhyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.wifi_name())
    }
}

/// WiFi frequency band
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Band {
    /// 2.4 GHz band (2400-2500 MHz)
    #[serde(rename = "2.4GHz")]
    Band2_4Ghz,
    /// 5 GHz band (5150-5895 MHz)
    #[serde(rename = "5GHz")]
    Band5Ghz,
    /// 6 GHz band (5925-7125 MHz, WiFi 6E/7)
    #[serde(rename = "6GHz")]
    Band6Ghz,
}

impl Band {
    /// Determine band from frequency in MHz
    pub fn from_frequency(freq_mhz: u32) -> Option<Self> {
        match freq_mhz {
            2400..=2500 => Some(Self::Band2_4Ghz),
            5150..=5895 => Some(Self::Band5Ghz),
            5925..=7125 => Some(Self::Band6Ghz),
            _ => None,
        }
    }
}

impl std::fmt::Display for Band {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Band2_4Ghz => write!(f, "2.4GHz"),
            Self::Band5Ghz => write!(f, "5GHz"),
            Self::Band6Ghz => write!(f, "6GHz"),
        }
    }
}

/// Channel information
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ChannelInfo {
    /// Center frequency in MHz
    pub frequency_mhz: u32,
    /// Channel number (calculated from frequency)
    pub channel: Option<u8>,
    /// Channel bandwidth in MHz (20, 40, 80, 160, 320)
    pub bandwidth_mhz: Option<u16>,
}

impl ChannelInfo {
    /// Get the frequency band
    pub fn band(&self) -> Option<Band> {
        Band::from_frequency(self.frequency_mhz)
    }

    fn freq_to_channel(f: u32) -> Option<u8> {
        match f {
            2412..=2472 => Some(((f - 2407) / 5) as u8),
            2484 => Some(14),
            5170..=5330 | 5490..=5895 => Some(((f - 5000) / 5) as u8),
            5935 => Some(2),
            5955..=7115 => Some(((f - 5950) / 5) as u8),
            _ => None,
        }
    }
}

/// BSS (Basic Service Set) operating mode
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum BssType {
    /// Infrastructure mode (has AP)
    Infrastructure,
    /// Independent BSS (ad-hoc)
    Ibss,
    /// Mesh network
    Mesh,
}

/// Capability flags from 802.11 beacon/probe response
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
pub struct Capabilities {
    /// ESS (Infrastructure mode)
    pub ess: bool,
    /// IBSS (Ad-hoc mode)
    pub ibss: bool,
    /// Privacy (encryption required)
    pub privacy: bool,
    /// Short preamble supported
    pub short_preamble: bool,
    /// Short slot time (802.11g)
    pub short_slot_time: bool,
    /// Spectrum management
    pub spectrum_mgmt: bool,
    /// Radio measurement
    pub radio_measure: bool,
    /// APSD (power save)
    pub apsd: bool,
}

impl Capabilities {
    /// Parse from 16-bit capability info field
    pub fn from_bits(bits: u16) -> Self {
        Self {
            ess: bits & 0x0001 != 0,
            ibss: bits & 0x0002 != 0,
            privacy: bits & 0x0010 != 0,
            short_preamble: bits & 0x0020 != 0,
            short_slot_time: bits & 0x0400 != 0,
            spectrum_mgmt: bits & 0x0100 != 0,
            radio_measure: bits & 0x1000 != 0,
            apsd: bits & 0x0800 != 0,
        }
    }

    /// Get BSS type from capabilities
    pub fn bss_type(&self) -> BssType {
        if self.ibss {
            BssType::Ibss
        } else {
            BssType::Infrastructure
        }
    }
}

/// Complete WiFi BSS (Access Point) information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WifiBssid {
    /// Network name (None for hidden networks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssid: Option<String>,
    /// MAC address of the access point
    #[serde(rename = "macAddress")]
    pub bssid: MacAddress,
    /// Time since last seen in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<u64>,
    /// Channel information
    pub channel_info: ChannelInfo,
    /// Physical layer types supported (newest first)
    #[serde(rename = "radioType")]
    pub phy: Vec<PhyType>,
    /// Signal strength in dBm
    #[serde(rename = "signalStrength")]
    pub rssi: i32,
    /// Security configuration
    pub wifi_security: Security,
    /// Raw capability strings
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub capabilities: Vec<String>,
}

impl WifiBssid {
    /// Get the highest supported PHY type
    pub fn best_phy(&self) -> PhyType {
        self.phy.first().copied().unwrap_or(PhyType::Legacy)
    }

    /// Check if network is hidden (no SSID broadcast)
    pub fn is_hidden(&self) -> bool {
        self.ssid.is_none()
    }

    /// Check if network is open (no encryption)
    pub fn is_open(&self) -> bool {
        self.wifi_security.version.is_none()
    }

    /// Get signal quality as percentage (approximate)
    pub fn signal_quality(&self) -> u8 {
        // Convert dBm to percentage (rough approximation)
        // -30 dBm = 100%, -90 dBm = 0%
        let clamped = self.rssi.clamp(-90, -30);
        ((clamped + 90) * 100 / 60) as u8
    }

    /// Get the frequency band
    pub fn band(&self) -> Option<Band> {
        self.channel_info.band()
    }
}

/// Signal strength classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalStrength {
    /// Excellent (-30 to -50 dBm)
    Excellent,
    /// Good (-50 to -60 dBm)
    Good,
    /// Fair (-60 to -70 dBm)
    Fair,
    /// Weak (-70 to -80 dBm)
    Weak,
    /// Poor (below -80 dBm)
    Poor,
}

impl SignalStrength {
    /// Classify signal strength from dBm value
    pub fn from_rssi(rssi: i32) -> Self {
        match rssi {
            -50..=0 => Self::Excellent,
            -60..=-51 => Self::Good,
            -70..=-61 => Self::Fair,
            -80..=-71 => Self::Weak,
            _ => Self::Poor,
        }
    }
}

impl std::fmt::Display for SignalStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Excellent => write!(f, "Excellent"),
            Self::Good => write!(f, "Good"),
            Self::Fair => write!(f, "Fair"),
            Self::Weak => write!(f, "Weak"),
            Self::Poor => write!(f, "Poor"),
        }
    }
}
