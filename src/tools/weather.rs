// https://api.met.no/doc/locationforecast/HowTO

use std::env;

use crate::{app_error::AppError, util::require_non_emtpy};

/// https://nominatim.openstreetmap.org/search?q=Oslo,Norway&format=json&limit=1
/// https://api.met.no/weatherapi/locationforecast/2.0/compact?lat=59.9133301&lon=10.7389701
/// /// USER AGENT!!

#[derive(Debug)]
pub struct WeatherClient {
    client: reqwest::Client,
    coord_base_url: String,
    weather_base_url: String,
    github_url: String,
}

impl WeatherClient {
    pub fn new(client: reqwest::Client) -> Result<Self, AppError> {
        let coord_base_url = require_non_emtpy("COORD_CLIENT_BASE_URL");
        let weather_base_url = require_non_emtpy("WEATHER_CLIENT_BASE_URL");
        let github_url = require_non_emtpy("GITHUB_URL");

        Ok(Self {
            client,
            coord_base_url,
            weather_base_url,
            github_url,
        })
    }

    async fn get_coordinates(&self, city: &str, country: &str) -> Result<(f32, f32), AppError> {
        let url = format!(
            "{}/search?q={},{}&format=json&limit=1",
            self.coord_base_url, city, country
        );
        let response = self
            .client
            .get(&url)
            .header("User-Agent", self.github_url.clone())
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or("Empty body".to_string());
            return Err(AppError::Http(status, body));
        }

        let body_text = response.text().await?;
        println!("JSON Body: {}", body_text);

        Ok((1.0, 1.0))
    }
}

#[cfg(test)]
pub mod tests {
    use dotenvy::dotenv;

    use crate::tools::weather::WeatherClient;

    #[tokio::test]
    async fn get_coordinates_success() {
        dotenv().ok();
        let client = WeatherClient::new(reqwest::Client::new()).unwrap();
        let result = client.get_coordinates("Oslo", "Norway").await.unwrap();
    }
}
