//! Configuration, constants, and TLS certificate management

use crate::error::{InterfaceError, InterfaceErrorKind};
use crate::scanner::wifi;
use clap::Parser;
use directories::ProjectDirs;
use neli_wifi::Nl80211Iftype as IfType;
use nix::net::if_::if_nametoindex;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::ffi::CString;
use std::sync::OnceLock;
use std::{error::Error, fs, path::PathBuf, process::Command};

/// Global configuration storage
static CONFIG: OnceLock<Config> = OnceLock::new();

/// Initialize the global configuration
pub fn init_config(config: Config) {
    CONFIG.set(config).expect("Config already initialized");
}

/// Get a reference to the global configuration.
pub fn get_config() -> &'static Config {
    CONFIG.get().expect("Config not initialized")
}

pub const SCAN_DURATION_SECS: u64 = 20; // Do not run longer than 29 seconds to avoid expired wifi scan results
pub const DWELL_TIME: u64 = 200; // in Time Units (1024 microseconds) - i think around 200ms per channel is a good balance
pub const GEOSUBMIT_ENDPOINT: &str = "https://api.beacondb.net/v2/geosubmit";
pub const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")); // not sure if this is the correct convention - todo
pub const MDNS_SERVICE_TYPE: &str = "Serviceberry";
pub const HTTP_SERVER_PORT: u16 = 8080;
pub const HTTPS_SERVER_PORT: u16 = 8443;

#[derive(Debug, Clone)]
pub struct Interface {
    pub index: u32,
    pub name: String,
}

/// CLI arguments - may be incomplete/unresolved
#[derive(Parser, Debug, Clone)]
#[command(name = "serviceberry")]
#[command(about = "Geolocation service via WiFi & Bluetooth scanning")]
pub struct ConfigArgs {
    /// Wi-Fi adapter to use (e.g., wlan0). Auto-detected if not specified.
    #[arg(value_name = "WIFI_ADAPTER")]
    pub wifi_adapter_name: Option<String>,

    #[arg(long, default_value_t = HTTP_SERVER_PORT)]
    pub http_server_port: u16,

    #[arg(long, default_value_t = HTTPS_SERVER_PORT)]
    pub https_server_port: u16,

    #[arg(long, default_value_t = SCAN_DURATION_SECS)]
    pub scan_duration_secs: u64,

    #[arg(long, default_value_t = DWELL_TIME)]
    pub dwell_time: u64,

    #[arg(long, default_value_t = GEOSUBMIT_ENDPOINT.to_string())]
    pub geosubmit_endpoint: String,

    #[arg(long, default_value_t = APP_USER_AGENT.to_string())]
    pub user_agent: String,

    /// Optional override for config directory
    #[arg(long)]
    pub directory: Option<PathBuf>,
}

/// Fully resolved configuration - guaranteed valid
#[derive(Debug, Clone)]
pub struct Config {
    pub interface: Interface,
    pub http_server_port: u16,
    pub https_server_port: u16,
    pub scan_duration_secs: u64,
    pub dwell_time: u64,
    pub geosubmit_endpoint: String,
    pub user_agent: String,
    pub directory: PathBuf,
}

impl ConfigArgs {
    pub fn parse_args() -> Self {
        let mut cfg = Self::parse();

        // If wifi_adapter not provided, try to auto detect
        if cfg.wifi_adapter_name.is_none() {
            cfg.wifi_adapter_name = Self::detect_default_adapter();
            if cfg.wifi_adapter_name.is_none() {
                eprintln!("No Wi-Fi adapter specified and none detected. Please provide one.");
                std::process::exit(1);
            }
        }

        cfg
    }

    /// Resolve CLI args into a fully validated Config
    pub fn resolve(self) -> Result<Config, InterfaceError> {
        let interface = self.resolve_interface()?;
        let directory = self.directory.unwrap_or_else(Config::default_config_dir);

        Ok(Config {
            interface,
            http_server_port: self.http_server_port,
            https_server_port: self.https_server_port,
            scan_duration_secs: self.scan_duration_secs,
            dwell_time: self.dwell_time,
            geosubmit_endpoint: self.geosubmit_endpoint,
            user_agent: self.user_agent,
            directory,
        })
    }

    fn resolve_interface(&self) -> Result<Interface, InterfaceError> {
        let name = self
            .wifi_adapter_name
            .as_deref()
            .ok_or_else(|| InterfaceError {
                kind: InterfaceErrorKind::NotFound,
                message: "no wifi adapter name set".into(),
            })?;

        let c_name = CString::new(name).map_err(|_| InterfaceError {
            kind: InterfaceErrorKind::InvalidName,
            message: format!("invalid interface name: {}", name),
        })?;

        match if_nametoindex(c_name.as_c_str()) {
            Ok(index) => Ok(Interface {
                index,
                name: name.to_string(),
            }),
            Err(_) => Err(InterfaceError {
                kind: InterfaceErrorKind::NotFound,
                message: format!("interface not found: {}", name),
            }),
        }
    }

    pub fn detect_default_adapter() -> Option<String> {
        let interfaces = wifi::fetch_wifi_interfaces().ok()?;

        for iface in interfaces {
            if let (Some(name_bytes), Some(iftype)) = (&iface.name, iface.iftype) {
                // Remove trailing null bytes (if any) from the interface name
                let name = name_bytes
                    .iter()
                    .take_while(|&&b| b != 0)
                    .cloned()
                    .collect::<Vec<u8>>();
                let name = String::from_utf8_lossy(&name).to_string();
                if iftype == IfType::IftypeStation && !name.is_empty() {
                    return Some(name);
                }
            }
        }
        None
    }
}

impl Config {
    pub fn default_config_dir() -> PathBuf {
        let proj_dirs = ProjectDirs::from("com", "LimesKey", "Serviceberry").unwrap();
        let dir = proj_dirs.config_dir();
        std::fs::create_dir_all(dir).expect(&format!(
            "Failed to create config directory at {}",
            dir.display()
        ));
        dir.to_path_buf()
    }
}

pub struct Identity {
    pub certs: Vec<CertificateDer<'static>>,
    pub key: PrivateKeyDer<'static>,
    pub certs_hash: [u8; 32],
}

// use mkcert for a locally trusted certificate - automatically valid in browser
pub fn gen_cert(
    hostname: &str,
    config_directory: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let cert_path = config_directory.join("cert.pem");
    let key_path = config_directory.join("key.pem");

    // only generate if the files don't exist yet
    if !cert_path.exists() || !key_path.exists() {
        // mkcert arguments: output files and hostnames
        let output = Command::new("mkcert")
            .args(&[
                "-cert-file",
                cert_path.to_str().unwrap(),
                "-key-file",
                key_path.to_str().unwrap(),
                hostname,
                // let mdns_hostname = format!(
                //     "{}-{}.local",
                //     config::MDNS_SERVICE_TYPE.to_lowercase(),
                //     username.to_lowercase()
                // );
            ])
            .output()?;

        if !output.status.success() {
            return Err(
                format!("mkcert failed: {}", String::from_utf8_lossy(&output.stderr)).into(),
            );
        }

        println!("Generated certificate and key with mkcert");
    } else {
        println!("Certificate and key already exist, skipping generation");
    }

    Ok(())
}

/// Load TLS identity from certificate and key files
pub fn load_identity(
    hostname: &str,
    config_directory: &PathBuf,
) -> Result<Identity, Box<dyn Error>> {
    let cert_path = config_directory.join("cert.pem");
    let key_path = config_directory.join("key.pem");

    if !std::path::Path::new(&cert_path).exists() || !std::path::Path::new(&key_path).exists() {
        // create keypair if not exist
        gen_cert(hostname, config_directory)?;
    }

    let certs = fs::read(cert_path)?;
    let keys = fs::read(key_path)?;

    let cert_content: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut &*certs) // load cert from file into PEM format
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(CertificateDer::from)
        .collect();
    let key_content = PrivateKeyDer::from(
        // load key from file into PEM format
        rustls_pemfile::pkcs8_private_keys(&mut &*keys)
            .collect::<Result<Vec<_>, _>>()?
            .pop()
            .ok_or("No private key found")?,
    );

    Ok(Identity::new(cert_content, key_content)?)
}

impl Identity {
    pub fn new(
        certs: Vec<CertificateDer<'static>>,
        key: PrivateKeyDer<'static>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut identity = Identity {
            certs,
            key,
            certs_hash: [0u8; 32],
        };
        identity.certs_hash = identity.fingerprint_sha256()?;
        Ok(identity)
    }

    /// Get SHA256 fingerprint of the certificate
    fn fingerprint_sha256(&self) -> Result<[u8; 32], Box<dyn std::error::Error>> {
        use sha2::{Digest, Sha256};

        let cert = self
            .certs
            .get(0)
            .ok_or("No certificates available for fingerprint")?;

        let mut hasher = Sha256::new();
        hasher.update(cert.as_ref());

        Ok(hasher.finalize().into())
    }
}
