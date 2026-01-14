use std::time::Duration;

use dotenvy::dotenv;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    tmp_light_api::server::start_server,
    tools::{transit::TransitClient, weather::WeatherClient, zigbee::ZigbeeController},
};

mod app_error;
mod audio;
mod logger;
mod mcp;
mod orchestrator;
mod tools;
mod util;

// TODO remove when finsihed
mod tmp_light_api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting son-of-anton");

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    /*
    let weather_client = WeatherClient::new(http_client.clone())?;
    let transit_client = TransitClient::new(http_client.clone());
    */
    let zigbee_controller = ZigbeeController::new("localhost", 1883).await;
    start_server(zigbee_controller).await?;

    Ok(())
}
