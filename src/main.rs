use std::time::Duration;

use dotenvy::dotenv;

use crate::tools::{transit::TransitClient, weather::WeatherClient, zigbee::ZigbeeController};

mod app_error;
mod logger;
mod mcp;
mod orchestrator;
mod tools;
mod util;
mod voice;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let weather_client = WeatherClient::new(http_client.clone())?;
    let transit_client = TransitClient::new(http_client.clone());
    let zigbee_controller = ZigbeeController::new("localhost", 1883).await;

    Ok(())
}
