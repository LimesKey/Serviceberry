use axum::{
    Json, Router,
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
};
use local_ip_address::local_ip;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::{collections::HashMap, net::SocketAddr, time::Duration};
use tokio::time::timeout;
use tower_http::trace::TraceLayer;
use tracing::Span;

mod adapters;
mod geosubmit;
use serde::{Deserialize, Serialize};

use crate::geosubmit::{SubmitError, assemble_geo_payload, items};

const SCAN_DURATION_SECS: u64 = 10;
const GEOSUBMIT_ENDPOINT: &str = "https://api.beacondb.net/v2/geosubmit";

#[tokio::main]
async fn main() {
    let version = env!("CARGO_PKG_VERSION");

    let service_type = "Serviceberry".to_lowercase(); // Service your running, ServiceBerry in this case
    let instance_name = "Home Server"; // pretty, human readable name for the device you're using
    let hostname = "limeskey"; // actual mDNS url name you're broadcasting as / second level domain
    let lan_ip = local_ip().expect("Could not get local IP address");
    let port = 8080;
    let properties = HashMap::from([
        ("version".into(), version.into()),
        ("paths".into(), "/submit, /status, /request".into()),
    ]);

    let service_info: ServiceInfo = ServiceInfo::new(
        &format!("_{}._tcp.local.", service_type),
        instance_name,
        &format!("{}.local.", hostname),
        &lan_ip.to_string(),
        port,
        Some(properties),
    )
    .expect("Failed to create service info");

    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    mdns.register(service_info)
        .expect("Failed to register mDNS service");

    println!(
        "mDNS service published as {} at {}:{}",
        instance_name,
        &lan_ip.to_string(),
        port
    );

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/submit", post(process_submit))
        .route("/status", get(handle_status))
        .route("/request", get(handle_request))
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
        );

    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct PartialPayload {
    position: serde_json::Value,
    cell_towers: Option<serde_json::Value>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

async fn process_submit(Json(payload): Json<serde_json::Value>) -> Result<String, StatusCode> {
    println!("[Server] /submit request received");

    println!("\n================ RECEIVED JSON =================");
    println!("{}", &payload);
    println!("============================================\n");

    let gps_response: PartialPayload = serde_json::from_value(payload).unwrap();
    let response = assemble_geo_payload(gps_response)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let handle = tokio::spawn(async move { items::submit_geo_payload(response.clone()).await });

    match timeout(Duration::from_secs(3), handle).await {
        Ok(join_result) => match join_result {
            Ok(Ok(())) => {
                println!("Successfully sent to {}", GEOSUBMIT_ENDPOINT);
            }
            Ok(Err(e)) => match *e {
                SubmitError::HttpStatus { status, ref body } => {
                    eprintln!("HTTP error {},\n {}", status, body);
                }
                SubmitError::Transport(ref err) => {
                    eprintln!("Transport error: {:?}", err);
                }
            },
            Err(join_err) => {
                eprintln!("Task panicked: {:?}", join_err);
            }
        },

        Err(_) => {
            tracing::debug!("Request took too long, not waiting for status...");
        }
    }

    Ok(String::from("Successful")) // add more detailed response later
}

async fn handle_status() -> Result<String, StatusCode> {
    Ok(String::from("todo")) // add more detailed response later
}

async fn handle_request() -> Result<String, StatusCode> {
    Ok(String::from("todo")) // add more detailed response later
}
