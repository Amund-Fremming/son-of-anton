use crate::{app_error::AppError, util::require_non_emtpy};
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Deserializer, Serialize, de};

/// https://github.com/metno/weathericons/tree/main/weather (icons for symbol_code)

/*
    TODO
    - get temperature now
    - get weather report for next 6 hours
    - get weather report for next 12 hours
    - get weather report for day x from 0800-2000 (12hours) some start temp, max, and progress with rain and so on
*/

/// Weather symbol codes from Yr API
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum WeatherSymbol {
    ClearSky,
    Fair,
    PartlyCloudy,
    Cloudy,
    LightRainShowers,
    RainShowers,
    HeavyRainShowers,
    LightRainShowersAndThunder,
    RainShowersAndThunder,
    HeavyRainShowersAndThunder,
    LightSleetShowers,
    SleetShowers,
    HeavySleetShowers,
    LightSleetShowersAndThunder,
    SleetShowersAndThunder,
    HeavySleetShowersAndThunder,
    LightSnowShowers,
    SnowShowers,
    HeavySnowShowers,
    LightSnowShowersAndThunder,
    SnowShowersAndThunder,
    HeavySnowShowersAndThunder,
    LightRain,
    Rain,
    HeavyRain,
    LightRainAndThunder,
    RainAndThunder,
    HeavyRainAndThunder,
    LightSleet,
    Sleet,
    HeavySleet,
    LightSleetAndThunder,
    SleetAndThunder,
    HeavySleetAndThunder,
    LightSnow,
    Snow,
    HeavySnow,
    LightSnowAndThunder,
    SnowAndThunder,
    HeavySnowAndThunder,
    Fog,
}

impl WeatherSymbol {
    pub fn as_norwegian(&self) -> &'static str {
        match self {
            Self::ClearSky => "klarvær",
            Self::Fair => "lettskyet",
            Self::PartlyCloudy => "delvis skyet",
            Self::Cloudy => "skyet",
            Self::LightRainShowers => "lette regnbyger",
            Self::RainShowers => "regnbyger",
            Self::HeavyRainShowers => "kraftige regnbyger",
            Self::LightRainShowersAndThunder => "lette regnbyger og torden",
            Self::RainShowersAndThunder => "regnbyger og torden",
            Self::HeavyRainShowersAndThunder => "kraftige regnbyger og torden",
            Self::LightSleetShowers => "lette sluddbyger",
            Self::SleetShowers => "sluddbyger",
            Self::HeavySleetShowers => "kraftige sluddbyger",
            Self::LightSleetShowersAndThunder => "lette sluddbyger og torden",
            Self::SleetShowersAndThunder => "sluddbyger og torden",
            Self::HeavySleetShowersAndThunder => "kraftige sluddbyger og torden",
            Self::LightSnowShowers => "lette snøbyger",
            Self::SnowShowers => "snøbyger",
            Self::HeavySnowShowers => "kraftige snøbyger",
            Self::LightSnowShowersAndThunder => "lette snøbyger og torden",
            Self::SnowShowersAndThunder => "snøbyger og torden",
            Self::HeavySnowShowersAndThunder => "kraftige snøbyger og torden",
            Self::LightRain => "lett regn",
            Self::Rain => "regn",
            Self::HeavyRain => "kraftig regn",
            Self::LightRainAndThunder => "lett regn og torden",
            Self::RainAndThunder => "regn og torden",
            Self::HeavyRainAndThunder => "kraftig regn og torden",
            Self::LightSleet => "lett sludd",
            Self::Sleet => "sludd",
            Self::HeavySleet => "kraftig sludd",
            Self::LightSleetAndThunder => "lett sludd og torden",
            Self::SleetAndThunder => "sludd og torden",
            Self::HeavySleetAndThunder => "kraftig sludd og torden",
            Self::LightSnow => "lett snø",
            Self::Snow => "snø",
            Self::HeavySnow => "kraftig snø",
            Self::LightSnowAndThunder => "lett snø og torden",
            Self::SnowAndThunder => "snø og torden",
            Self::HeavySnowAndThunder => "kraftig snø og torden",
            Self::Fog => "tåke",
        }
    }
}

impl<'de> Deserialize<'de> for WeatherSymbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let base = s
            .trim_end_matches("_day")
            .trim_end_matches("_night")
            .trim_end_matches("_polartwilight");

        match base {
            "clearsky" => Ok(Self::ClearSky),
            "fair" => Ok(Self::Fair),
            "partlycloudy" => Ok(Self::PartlyCloudy),
            "cloudy" => Ok(Self::Cloudy),
            "lightrainshowers" => Ok(Self::LightRainShowers),
            "rainshowers" => Ok(Self::RainShowers),
            "heavyrainshowers" => Ok(Self::HeavyRainShowers),
            "lightrainshowersandthunder" => Ok(Self::LightRainShowersAndThunder),
            "rainshowersandthunder" => Ok(Self::RainShowersAndThunder),
            "heavyrainshowersandthunder" => Ok(Self::HeavyRainShowersAndThunder),
            "lightsleetshowers" => Ok(Self::LightSleetShowers),
            "sleetshowers" => Ok(Self::SleetShowers),
            "heavysleetshowers" => Ok(Self::HeavySleetShowers),
            "lightssleetshowersandthunder" | "lightsleetshowersandthunder" => {
                Ok(Self::LightSleetShowersAndThunder)
            }
            "sleetshowersandthunder" => Ok(Self::SleetShowersAndThunder),
            "heavysleetshowersandthunder" => Ok(Self::HeavySleetShowersAndThunder),
            "lightsnowshowers" => Ok(Self::LightSnowShowers),
            "snowshowers" => Ok(Self::SnowShowers),
            "heavysnowshowers" => Ok(Self::HeavySnowShowers),
            "lightssnowshowersandthunder" | "lightsnowshowersandthunder" => {
                Ok(Self::LightSnowShowersAndThunder)
            }
            "snowshowersandthunder" => Ok(Self::SnowShowersAndThunder),
            "heavysnowshowersandthunder" => Ok(Self::HeavySnowShowersAndThunder),
            "lightrain" => Ok(Self::LightRain),
            "rain" => Ok(Self::Rain),
            "heavyrain" => Ok(Self::HeavyRain),
            "lightrainandthunder" => Ok(Self::LightRainAndThunder),
            "rainandthunder" => Ok(Self::RainAndThunder),
            "heavyrainandthunder" => Ok(Self::HeavyRainAndThunder),
            "lightsleet" => Ok(Self::LightSleet),
            "sleet" => Ok(Self::Sleet),
            "heavysleet" => Ok(Self::HeavySleet),
            "lightsleetandthunder" => Ok(Self::LightSleetAndThunder),
            "sleetandthunder" => Ok(Self::SleetAndThunder),
            "heavysleetandthunder" => Ok(Self::HeavySleetAndThunder),
            "lightsnow" => Ok(Self::LightSnow),
            "snow" => Ok(Self::Snow),
            "heavysnow" => Ok(Self::HeavySnow),
            "lightsnowandthunder" => Ok(Self::LightSnowAndThunder),
            "snowandthunder" => Ok(Self::SnowAndThunder),
            "heavysnowandthunder" => Ok(Self::HeavySnowAndThunder),
            "fog" => Ok(Self::Fog),
            _ => Err(de::Error::custom(format!("unknown symbol_code: {}", s))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct YrResponse {
    properties: YrProperties,
}

#[derive(Debug, Serialize, Deserialize)]
struct YrProperties {
    timeseries: Vec<Forecast>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Forecast {
    time: DateTime<Utc>,
    data: ForecastData,
}

#[derive(Debug, Serialize, Deserialize)]
struct ForecastData {
    instant: CurrentWeather,
    next_1_hours: Option<ForecastPeriod>,
    next_6_hours: Option<ForecastPeriod>,
    next_12_hours: Option<ForecastPeriod>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CurrentWeather {
    details: WeatherDetails,
}

#[derive(Debug, Serialize, Deserialize)]
struct WeatherDetails {
    air_temperature: f32,
    wind_speed: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ForecastPeriod {
    summary: WeatherCondition,
    details: Precipitation,
}

#[derive(Debug, Serialize, Deserialize)]
struct WeatherCondition {
    symbol_code: WeatherSymbol,
}

#[derive(Debug, Serialize, Deserialize)]
struct Precipitation {
    precipitation_amount: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
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

        let response: YrResponse = response.json().await?;

        println!("{}", serde_json::to_string_pretty(&response).unwrap());

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
    }
}
