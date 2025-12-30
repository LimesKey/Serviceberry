//! Configuration, constants, and TLS certificate management

use directories::ProjectDirs;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::{error::Error, fs, path::PathBuf, process::Command};

pub const SCAN_DURATION_SECS: u64 = 10;
pub const GEOSUBMIT_ENDPOINT: &str = "https://api.beacondb.net/v2/geosubmit";
pub const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")); // not sure if this is the correct convention - todo
pub const MDNS_SERVICE_TYPE: &str = "Serviceberry";
pub const HTTP_SERVER_PORT: u16 = 8080;
pub const HTTPS_SERVER_PORT: u16 = 8443;

/// Get the project configuration directory
pub fn config_dir() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "LimesKey", "serviceberry")
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

// use mkcert for a locally trusted certificate - automatically valid in browser
pub fn gen_cert(
    hostname: &String,
    config_directory: PathBuf,
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
    hostname: &String,
    config_directory: PathBuf,
) -> Result<Identity, Box<dyn Error>> {
    let cert_path = config_directory.join("cert.pem");
    let key_path = config_directory.join("key.pem");

    if !std::path::Path::new(&cert_path).exists() || !std::path::Path::new(&key_path).exists() {
        // create keypair if not exist
        gen_cert(hostname, config_directory.clone())?;
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
