use dotenvy::dotenv;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::tools::{util::require_non_emtpy, zigbee::ZigbeeController};

mod tools;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting son-of-anton");

    let sleep_duration: u64 = require_non_emtpy("SLEEP_DURATION").parse()?;
    let _zigbee_controller = ZigbeeController::new("localhost", 1883, sleep_duration).await;

    info!("Controller initialized. Listening for button presses... Press Ctrl+C to exit.");
    
    // Keep the application running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}
