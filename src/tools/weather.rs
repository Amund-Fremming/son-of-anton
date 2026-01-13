use crate::{app_error::AppError, util::require_non_emtpy};
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Coordinates {
    lat: String,
    lon: String,
}

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

    async fn get_coordinates(
        &self,
        city: &str,
        country: &str,
    ) -> Result<(String, String), AppError> {
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

        let coords: Vec<Coordinates> = response.json().await?;
        let coord = coords.into_iter().next().ok_or_else(|| {
            AppError::Http(
                StatusCode::NOT_FOUND,
                "Coordinates not found for location".to_string(),
            )
        })?;

        Ok((coord.lat, coord.lon))
    }

    pub async fn get_weather(&self, city: &str, country: &str) -> Result<(), AppError> {
        let (lat, lon) = self.get_coordinates(city, country).await?;
        let url = format!("{}/compact?lat={}&lon={}", self.weather_base_url, lat, lon);
        let response = self
            .client
            .get(url)
            .header("User-Agent", self.github_url.clone())
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or("Empty body".to_string());
            return Err(AppError::Http(status, body));
        }

        let json = response.text().await?;
        println!("JSON: {}", json);

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::tools::weather::WeatherClient;
    use dotenvy::dotenv;

    fn setup_client() -> WeatherClient {
        dotenv().ok();
        let result = WeatherClient::new(reqwest::Client::new());
        assert!(
            result.is_ok(),
            "Failed to create weather client: {}",
            result.err().unwrap().to_string()
        );

        result.unwrap()
    }

    #[tokio::test]
    async fn get_coordinates_success() {
        let client = setup_client();
        let result = client.get_coordinates("Oslo", "Norway").await;
        assert!(
            result.is_ok(),
            "Client request failed: {}",
            result.err().unwrap().to_string()
        );

        let (lat, lon) = result.unwrap();
        assert_eq!("59.9133301", lat);
        assert_eq!("10.7389701", lon);
    }

    #[tokio::test]
    async fn get_weather_success() {
        let client = setup_client();
        let result = client.get_weather("Oslo", "Norway").await;
        assert!(
            result.is_ok(),
            "Client request failed: {}",
            result.err().unwrap().to_string()
        );

        println!()
    }
}
