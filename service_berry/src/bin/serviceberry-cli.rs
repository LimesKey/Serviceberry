use service_berry::{self, config::ConfigArgs};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigArgs::parse_args().resolve()?;
    tracing_subscriber::fmt::init();
    service_berry::run(config).await?;
    Ok(())
}
