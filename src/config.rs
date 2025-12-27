//! Configuration, constants, and TLS certificate management

use directories::ProjectDirs;
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::{error::Error, fs, path::PathBuf};

pub const SCAN_DURATION_SECS: u64 = 10;
pub const GEOSUBMIT_ENDPOINT: &str = "https://api.beacondb.net/v2/geosubmit";
pub const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
pub const MDNS_SERVICE_TYPE: &str = "Serviceberry";
pub const HTTP_SERVER_PORT: u16 = 8080;
pub const DEFAULT_HOSTNAME: &str = "turtle";
pub const MDNS_INSTANCE_NAME: &str = "Home Server";

/// Get the project configuration directory
pub fn config_dir() -> PathBuf {
    let proj_dirs = ProjectDirs::from("org", "LimesKey", "serviceberry")
        .expect("Failed to get project directories");

    let config_dir = proj_dirs.config_dir();
    fs::create_dir_all(config_dir).expect("Failed to create config directory");
    config_dir.to_path_buf()
}

pub struct Identity {
    pub certs: Vec<CertificateDer<'static>>,
    pub key: PrivateKeyDer<'static>,
    pub certs_hash: [u8; 32],
}

/// Generate self-signed certificate and key if they don't already exist in the config dir
pub fn gen_cert(
    hostname: String,
    config_directory: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let cert_path = config_directory.join("cert.pem");
    let key_path = config_directory.join("key.pem");

    let subject_alt_names = vec!["localhost".to_string(), format!("{}.local", hostname)];
    let cert_pair = generate_simple_self_signed(subject_alt_names)?;

    fs::write(&cert_path, cert_pair.cert.pem())?;
    fs::write(&key_path, cert_pair.signing_key.serialize_pem())?;

    println!("Generated self-signed certificate and key");
    Ok(())
}

/// Load TLS identity from certificate and key files
pub fn load_identity(
    hostname: String,
    config_directory: PathBuf,
) -> Result<Identity, Box<dyn Error>> {
    let cert_path = config_directory.join("cert.pem");
    let key_path = config_directory.join("key.pem");

    if !std::path::Path::new(&cert_path).exists() || !std::path::Path::new(&key_path).exists() {
        // create keypair if not exist
        gen_cert(hostname.clone(), config_directory.clone())?;
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
