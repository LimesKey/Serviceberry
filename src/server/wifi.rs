use rcgen::{Certificate, generate_simple_self_signed};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::path::Path;
use std::{fs, io::BufReader};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

/// Generate self-signed certificate and key if they don't exist.
/// Returns tuple of (cert_path, key_path)
pub fn gen_cert(hostname: String) -> Result<(String, String), Box<dyn std::error::Error>> {
    let cert_path = "cert.pem";
    let key_path = "key.pem";

    if !Path::new(cert_path).exists() || !Path::new(key_path).exists() {
        let subject_alt_names = vec!["localhost".to_string(), format!("{}.local", hostname)];
        let cert = generate_simple_self_signed(subject_alt_names)?;

        fs::write(cert_path, cert.cert.pem())?;
        fs::write(key_path, cert.signing_key.serialize_pem())?;

        println!("Generated self-signed certificate and key");
    } else {
        println!("Certificate and key already exist");
    }

    Ok((cert_path.to_string(), key_path.to_string()))
}

fn load_identity() -> Result<Identity> {
    let cert_path = "cert.pem";
    let key_path = "key.pem";

    let cert_bytes = fs::read(cert_path)?;
    let key_bytes  = fs::read(key_path)?;

    // parse certs
    let certs_der = certs(&mut Cursor::new(cert_bytes))?
        .into_iter()
        .map(|v| CertificateDer::from(v))
        .collect();

    // parse keys (PKCS#8)
    let mut keys = pkcs8_private_keys(&mut Cursor::new(key_bytes))?;
    let key_der = keys
        .pop()
        .ok_or_else(|| anyhow::anyhow!("no private key found"))?;
    let key_der = PrivateKeyDer::from(key_der);

    Ok(Identity {
        certs: certs_der,
        key: key_der,
    })
}


fn cert_fingerprint_sha256(path: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut certs = rustls_pemfile::certs(&mut reader);

    let cert = certs.next().ok_or("no certificate found")??;

    let mut hasher = Sha256::new();
    hasher.update(cert.as_ref());

    Ok(hasher.finalize().into())
}
