//! ServiceBerry - Geolocation service via WiFi & Bluetooth scanning
//!
//! A service that scans nearby WiFi and Bluetooth devices and submits
//! location data to the Ichnaea geolocation service.

pub mod config;
pub mod error;

pub mod scanner {
    pub mod bluetooth;
    pub mod wifi;

    pub use self::bluetooth::BleDevice;
    pub use self::wifi::WifiBssid;
}

pub mod geosubmit {
    pub mod client;
    pub mod payload;

    pub use self::client::{assemble_geo_payload, submit_geo_payload};
    pub use self::payload::{CellTower, Position, RadioType, items};
}

pub mod peripheral {
    pub mod gatt;

    pub use self::gatt::ble_peripheral;
}

pub mod server {
    pub mod geo_provider;
    pub mod handlers;
    pub mod mdns_service;

    use axum::routing::{get, options, post};
    use axum::{Router, body::Body, http::Request};
    use hyper_util::rt::tokio::TokioIo;
    use rustls::ServerConfig;
    use std::{net::SocketAddr, sync::Arc};
    use tokio::net::TcpListener;
    use tokio_rustls::TlsAcceptor;
    use tower_http::trace::TraceLayer;
    use tracing::{Span, debug, info};

    use crate::config::Identity;
    use crate::error::Result;

    pub fn create_router() -> Router {
        Router::new()
            .route("/submit", post(handlers::submit))
            .route("/status", get(handlers::status))
            .route("/request", post(handlers::request_post))
            .route("/request", options(handlers::request_options))
            .route("/", get(handlers::root))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &Request<Body>| {
                        let user_agent = request
                            .headers()
                            .get("user-agent")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("<unknown>");
                        let remote_addr = request
                            .extensions()
                            .get::<SocketAddr>()
                            .map(|sa| sa.ip().to_string())
                            .unwrap_or_else(|| "<unknown>".into());

                        tracing::info_span!(
                            "http-request",
                            method = %request.method(),
                            uri = %request.uri(),
                            user_agent = %user_agent,
                            remote_addr = %remote_addr,
                        )
                    })
                    .on_request(|request: &Request<Body>, _span: &Span| {
                        tracing::info!("started {} {}", request.method(), request.uri().path());
                    }),
            )
    }

    pub async fn start_https(identity: Identity, port: u16) -> Result<()> {
        let certs = identity.certs;
        let key = identity.key;

        let mut tls_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| crate::error::Error::Other(e.to_string()))?;

        tls_config.alpn_protocols = vec![b"http/1.1".to_vec()];

        debug!("HTTPS TLS configuration initialized");

        let acceptor = TlsAcceptor::from(Arc::new(tls_config));
        let listener = TcpListener::bind(("0.0.0.0", port))
            .await
            .map_err(|e| crate::error::Error::Bind(e.to_string()))?;
        info!("HTTPS server listening on 0.0.0.0:{}", port);

        let router = create_router();
        loop {
            let (stream, _) = listener
                .accept()
                .await
                .map_err(|e| crate::error::Error::Bind(e.to_string()))?;

            let acceptor = acceptor.clone();
            let router = router.clone();

            tokio::spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        let io = TokioIo::new(tls_stream);
                        let hyper_service = hyper_util::service::TowerToHyperService::new(router);

                        if let Err(e) = hyper::server::conn::http1::Builder::new()
                            .serve_connection(io, hyper_service)
                            .await
                        {
                            eprintln!("Connection error: {}", e);
                        }
                    }
                    Err(e) => eprintln!("TLS handshake error: {}", e),
                }
            });
        }
    }

    pub async fn start_http(port: u16) -> Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", port))
            .await
            .map_err(|e| crate::error::Error::Bind(e.to_string()))?;
        info!("HTTP server listening on 0.0.0.0:{}", port);

        let router = create_router();
        loop {
            let (stream, _) = listener
                .accept()
                .await
                .map_err(|e| crate::error::Error::Bind(e.to_string()))?;

            let router = router.clone();

            tokio::spawn(async move {
                let io = TokioIo::new(stream);
                let hyper_service = hyper_util::service::TowerToHyperService::new(router);

                if let Err(e) = hyper::server::conn::http1::Builder::new()
                    .serve_connection(io, hyper_service)
                    .await
                {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }
}

use local_ip_address::local_ip;
use tokio::sync::mpsc;
use tracing::info;
use users::get_current_username;

pub async fn run(config: config::Config) -> crate::error::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // get system info
    let hostname = hostname::get() // computer/device name, e.g: "My-MacBook"
        .unwrap_or_else(|_| "unknown-device-hostname".into())
        .to_string_lossy()
        .to_string();
    let username = get_current_username() // operating system username, e.g: "john"
        .expect("Cannot retrieve operating system username!")
        .to_string_lossy()
        .to_string();
    let version = env!("CARGO_PKG_VERSION");
    let lan_ip = local_ip().expect("Could not get local IP address");
    info!("Local IP address: {}", lan_ip);
    let mdns_hostname = format!(
        "{}-{}.local",
        config::MDNS_SERVICE_TYPE.to_lowercase(),
        username.to_lowercase()
    );

    info!("Starting ServiceBerry v{} on {}", version, hostname);

    // Generate TLS certificates
    let identity = config::load_identity(&mdns_hostname, &config.directory)
        .map_err(|e| crate::error::Error::Other(e.to_string()))?;

    // Register mDNS service
    let _mdns = server::mdns_service::register_mdns_service(&hostname, lan_ip, version, &username)
        .map_err(|e| crate::error::Error::Other(format!("Failed to register mDNS: {}", e)))?;

    let (tx, mut rx) = mpsc::channel::<PartialPayload>(100);

    // Start the BLE peripheral
    tokio::spawn(async move {
        peripheral::ble_peripheral(tx).await;
    });

    tokio::spawn(async move {
        while let Some(payload) = rx.recv().await {
            tokio::spawn(async move {
                tracing::debug!("Worker processing payload: {:?}", payload);
                if let Err(e) = server::handlers::submit_payload(payload).await {
                    tracing::error!("Failed to process submission: {:?}", e);
                }
            });
        }
    });

    tokio::spawn({
        let cfg = config.clone();
        async move {
            if let Err(e) = server::start_http(cfg.http_server_port).await {
                tracing::error!("HTTP server error: {:?}", e);
            }
        }
    });

    server::start_https(identity, config.https_server_port).await?;
    Ok(())
}

pub use error::{Error, Result};
pub use geosubmit::{CellTower, Position, items};
pub use scanner::{BleDevice, WifiBssid};

use crate::server::handlers::PartialPayload;
