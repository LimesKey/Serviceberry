use directories::ProjectDirs;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::{fs, path::PathBuf};

pub mod bluetooth;
pub mod wifi;

struct Identity {
    certs: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
    cert_path: PathBuf,
    key_path: PathBuf,
}

pub fn get_config_dir() -> PathBuf {
    // Use your domain, organization, and app name
    let proj_dirs = ProjectDirs::from("org", "LimesKey", "serviceberry")
        .expect("Failed to get project directories");

    let config_dir = proj_dirs.config_dir();

    // Create the directory if it doesn't exist
    fs::create_dir_all(config_dir).expect("Failed to create config directory");

    config_dir.to_path_buf()
}
