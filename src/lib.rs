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
    pub mod handlers;
    pub mod mdns_service;

    use axum::routing::{get, post};
    use axum::{Router, body::Body, http::Request};
    use hyper_util::rt::tokio::TokioIo;
    use rustls::ServerConfig;
    use std::{net::SocketAddr, sync::Arc};
    use tokio::net::TcpListener;
    use tokio_rustls::TlsAcceptor;
    use tower_http::trace::TraceLayer;
    use tracing::Span;

    use crate::config::{HTTP_SERVER_PORT, Identity};
    use crate::error::Result;

    pub fn create_router() -> Router {
        Router::new()
            .route("/submit", post(handlers::process_submit_http))
            .route("/status", get(handlers::handle_status))
            .route("/request", get(handlers::handle_request))
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

    pub async fn start_tls(identity: Identity) -> Result<()> {
        let certs = identity.certs;
        let key = identity.key;

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| crate::error::Error::Other(e.to_string()))?;

        let acceptor = TlsAcceptor::from(Arc::new(config));
        let listener = TcpListener::bind(("0.0.0.0", HTTP_SERVER_PORT))
            .await
            .map_err(|e| crate::error::Error::Bind(e.to_string()))?;

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
}

pub use error::{Error, Result};
pub use geosubmit::{CellTower, Position, items};
pub use scanner::{BleDevice, WifiBssid};
