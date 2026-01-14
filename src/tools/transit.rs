use crate::{app_error::AppError, util::require_non_emtpy};
use serde::{Deserialize, Serialize};

/// Entur GraphQL API client for Oslo public transit (trams, buses, metro)
/// API documentation: https://developer.entur.org/

// GraphQL request and response structures
#[derive(Debug, Serialize)]
struct GraphQLRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphQLResponse {
    data: Option<GraphQLData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphQLData {
    #[serde(rename = "stopPlace")]
    stop_place: Option<StopPlace>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StopPlace {
    name: String,
    #[serde(rename = "estimatedCalls")]
    estimated_calls: Vec<EstimatedCall>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EstimatedCall {
    #[serde(rename = "expectedDepartureTime")]
    expected_departure_time: String,
    #[serde(rename = "destinationDisplay")]
    destination_display: Option<DestinationDisplay>,
    #[serde(rename = "serviceJourney")]
    service_journey: ServiceJourney,
}

#[derive(Debug, Serialize, Deserialize)]
struct DestinationDisplay {
    #[serde(rename = "frontText")]
    front_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceJourney {
    line: Line,
}

#[derive(Debug, Serialize, Deserialize)]
struct Line {
    #[serde(rename = "publicCode")]
    public_code: String,
    #[serde(rename = "transportMode")]
    transport_mode: String,
}

// Journey planning structures
#[derive(Debug, Serialize, Deserialize)]
struct JourneyResponse {
    data: Option<JourneyData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JourneyData {
    trip: Option<TripResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TripResult {
    #[serde(rename = "tripPatterns")]
    trip_patterns: Vec<TripPattern>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TripPattern {
    duration: Option<i32>,
    #[serde(rename = "walkDistance")]
    walk_distance: Option<f32>,
    legs: Vec<Leg>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Leg {
    mode: String,
    #[serde(rename = "fromPlace")]
    from_place: Place,
    #[serde(rename = "toPlace")]
    to_place: Place,
    #[serde(rename = "expectedStartTime")]
    expected_start_time: String,
    #[serde(rename = "expectedEndTime")]
    expected_end_time: String,
    line: Option<LegLine>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Place {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LegLine {
    #[serde(rename = "publicCode")]
    public_code: Option<String>,
}

#[derive(Debug)]
pub struct TransitClient {
    client: reqwest::Client,
    base_url: String,
    github_url: String,
}

impl TransitClient {
    pub fn new(client: reqwest::Client) -> Result<Self, AppError> {
        let base_url = require_non_emtpy("TRANSIT_CLIENT_BASE_URL");
        let github_url = require_non_emtpy("GITHUB_URL");
        Ok(Self {
            client,
            base_url,
            github_url,
        })
    }

    /// Fetch tram departures from a specific stop in Oslo
    /// stop_place_id: NSR Stop Place ID (e.g., "NSR:StopPlace:58366" for Nationaltheatret)
    /// destination: Optional destination filter (e.g., "Majorstuen", "Ljabru")
    pub async fn get_tram_departures(
        &self,
        stop_place_id: &str,
        destination: Option<&str>,
    ) -> Result<(), AppError> {
        let query = format!(
            r#"{{
                stopPlace(id: "{}") {{
                    name
                    estimatedCalls(timeRange: 72100, numberOfDepartures: 10) {{
                        expectedDepartureTime
                        destinationDisplay {{
                            frontText
                        }}
                        serviceJourney {{
                            line {{
                                publicCode
                                transportMode
                            }}
                        }}
                    }}
                }}
            }}"#,
            stop_place_id
        );

        let request = GraphQLRequest {
            query,
            variables: None,
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("User-Agent", self.github_url.clone())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or("Empty body".to_string());
            return Err(AppError::Http(status, body));
        }

        let mut graphql_response: GraphQLResponse = response.json().await?;

        // Filter by transport mode (tram) and optionally by destination
        if let Some(data) = &mut graphql_response.data {
            if let Some(stop_place) = &mut data.stop_place {
                stop_place.estimated_calls.retain(|call| {
                    let is_tram = call.service_journey.line.transport_mode == "tram";
                    let matches_destination = if let Some(dest) = destination {
                        call.destination_display
                            .as_ref()
                            .map(|d| d.front_text.contains(dest))
                            .unwrap_or(false)
                    } else {
                        true
                    };
                    is_tram && matches_destination
                });
            }
        }

        println!(
            "{}",
            serde_json::to_string_pretty(&graphql_response).unwrap()
        );

        Ok(())
    }

    /// Fetch bus departures from a specific stop by stop place ID
    /// stop_place_id: NSR Stop Place ID (e.g., "NSR:StopPlace:58366")
    /// destination: Optional destination filter (e.g., "Sognsvann", "Tonsenhagen")
    pub async fn get_bus_departures_by_stop(
        &self,
        stop_place_id: &str,
        destination: Option<&str>,
    ) -> Result<(), AppError> {
        let query = format!(
            r#"{{
                stopPlace(id: "{}") {{
                    name
                    estimatedCalls(timeRange: 72100, numberOfDepartures: 10) {{
                        expectedDepartureTime
                        destinationDisplay {{
                            frontText
                        }}
                        serviceJourney {{
                            line {{
                                publicCode
                                transportMode
                            }}
                        }}
                    }}
                }}
            }}"#,
            stop_place_id
        );

        let request = GraphQLRequest {
            query,
            variables: None,
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("User-Agent", self.github_url.clone())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or("Empty body".to_string());
            return Err(AppError::Http(status, body));
        }

        let mut graphql_response: GraphQLResponse = response.json().await?;

        // Filter by transport mode (bus) and optionally by destination
        if let Some(data) = &mut graphql_response.data {
            if let Some(stop_place) = &mut data.stop_place {
                stop_place.estimated_calls.retain(|call| {
                    let is_bus = call.service_journey.line.transport_mode == "bus";
                    let matches_destination = if let Some(dest) = destination {
                        call.destination_display
                            .as_ref()
                            .map(|d| d.front_text.contains(dest))
                            .unwrap_or(false)
                    } else {
                        true
                    };
                    is_bus && matches_destination
                });
            }
        }

        println!(
            "{}",
            serde_json::to_string_pretty(&graphql_response).unwrap()
        );

        Ok(())
    }

    /// Fetch bus departures for a specific line number
    /// stop_place_id: NSR Stop Place ID where the bus stops
    /// line_number: Bus line number (e.g., "54", "37")
    /// destination: Optional destination filter
    pub async fn get_bus_departures_by_line(
        &self,
        stop_place_id: &str,
        line_number: &str,
        destination: Option<&str>,
    ) -> Result<(), AppError> {
        let query = format!(
            r#"{{
                stopPlace(id: "{}") {{
                    name
                    estimatedCalls(timeRange: 72100, numberOfDepartures: 10) {{
                        expectedDepartureTime
                        destinationDisplay {{
                            frontText
                        }}
                        serviceJourney {{
                            line {{
                                publicCode
                                transportMode
                            }}
                        }}
                    }}
                }}
            }}"#,
            stop_place_id
        );

        let request = GraphQLRequest {
            query,
            variables: None,
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("User-Agent", self.github_url.clone())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or("Empty body".to_string());
            return Err(AppError::Http(status, body));
        }

        let mut graphql_response: GraphQLResponse = response.json().await?;

        // Filter by transport mode (bus), line number, and optionally by destination
        if let Some(data) = &mut graphql_response.data {
            if let Some(stop_place) = &mut data.stop_place {
                stop_place.estimated_calls.retain(|call| {
                    let is_bus = call.service_journey.line.transport_mode == "bus";
                    let is_correct_line = call.service_journey.line.public_code == line_number;
                    let matches_destination = if let Some(dest) = destination {
                        call.destination_display
                            .as_ref()
                            .map(|d| d.front_text.contains(dest))
                            .unwrap_or(false)
                    } else {
                        true
                    };
                    is_bus && is_correct_line && matches_destination
                });
            }
        }

        println!(
            "{}",
            serde_json::to_string_pretty(&graphql_response).unwrap()
        );

        Ok(())
    }

    /// Plan a journey between two locations
    /// from: Starting location name (e.g., "Torshov", "Nationaltheatret")
    /// to: Destination location name (e.g., "Oslo S", "Majorstuen")
    pub async fn plan_journey(&self, from: &str, to: &str) -> Result<(), AppError> {
        let query = format!(
            r#"{{
                trip(
                    from: {{
                        name: "{}"
                    }}
                    to: {{
                        name: "{}"
                    }}
                    numTripPatterns: 5
                ) {{
                    tripPatterns {{
                        duration
                        walkDistance
                        legs {{
                            mode
                            fromPlace {{
                                name
                            }}
                            toPlace {{
                                name
                            }}
                            expectedStartTime
                            expectedEndTime
                            line {{
                                publicCode
                            }}
                        }}
                    }}
                }}
            }}"#,
            from, to
        );

        let request = GraphQLRequest {
            query,
            variables: None,
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("User-Agent", self.github_url.clone())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or("Empty body".to_string());
            return Err(AppError::Http(status, body));
        }

        let journey_response: JourneyResponse = response.json().await?;

        println!(
            "{}",
            serde_json::to_string_pretty(&journey_response).unwrap()
        );

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::TransitClient;
    use dotenvy::dotenv;

    fn setup_client() -> TransitClient {
        dotenv().ok();
        let result = TransitClient::new(reqwest::Client::new());
        assert!(
            result.is_ok(),
            "Failed to create transit client: {}",
            result.err().unwrap().to_string()
        );

        result.unwrap()
    }

    #[tokio::test]
    async fn get_tram_departures_success() {
        let client = setup_client();
        // NSR:StopPlace:58366 is Nationaltheatret
        let result = client
            .get_tram_departures("NSR:StopPlace:58366", None)
            .await;
        assert!(
            result.is_ok(),
            "Client request failed: {}",
            result.err().unwrap().to_string()
        );
    }

    #[tokio::test]
    async fn get_bus_departures_by_stop_success() {
        let client = setup_client();
        // NSR:StopPlace:58366 is Nationaltheatret
        let result = client
            .get_bus_departures_by_stop("NSR:StopPlace:58366", None)
            .await;
        assert!(
            result.is_ok(),
            "Client request failed: {}",
            result.err().unwrap().to_string()
        );
    }

    #[tokio::test]
    async fn get_bus_departures_by_line_success() {
        let client = setup_client();
        // NSR:StopPlace:58366 is Nationaltheatret, line 54 stops there
        let result = client
            .get_bus_departures_by_line("NSR:StopPlace:58366", "54", None)
            .await;
        assert!(
            result.is_ok(),
            "Client request failed: {}",
            result.err().unwrap().to_string()
        );
    }

    #[tokio::test]
    async fn plan_journey_success() {
        let client = setup_client();
        // Plan journey from Torshov to Oslo S
        let result = client.plan_journey("Torshov", "Birkelunden").await;
        assert!(
            result.is_ok(),
            "Client request failed: {}",
            result.err().unwrap().to_string()
        );
    }
}
