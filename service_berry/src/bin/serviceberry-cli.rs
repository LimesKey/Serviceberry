use service_berry::{self, config::Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse_args();

    tracing_subscriber::fmt::init();
    service_berry::run(config).await?;
    Ok(())
}