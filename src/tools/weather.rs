use anyhow::{Context, Result};
use reqwest_middleware::ClientWithMiddleware;
use rmcp::tool;
use std::time::Duration;
use tracing::{debug, instrument};

use crate::http::HTTP_CLIENT;
use crate::models::{
    GeocodingResponse, OpenMeteoResponse, TemperatureUnit, WeatherRequest, WeatherResponse,
};

/// Get current weather information for a specified location.
///
/// Returns temperature (automatically converted to Celsius or Fahrenheit based on the country),
/// WMO weather code, and wind speed in km/h. Supports city names, addresses, or place names worldwide.
///
/// # Errors
///
/// Returns an error if the geocoding or weather API calls fail.
#[tool(
    description = "Get current weather information for a specified location. Returns temperature (automatically converted to Celsius or Fahrenheit based on the country), WMO weather code, and wind speed in km/h. Supports city names, addresses, or place names worldwide."
)]
#[instrument(fields(location = %request.location))]
pub async fn get_weather(request: WeatherRequest) -> Result<WeatherResponse> {
    get_weather_with_client(&HTTP_CLIENT, request).await
}

/// Internal function that allows injecting an HTTP client and base URLs for testing
///
/// # Errors
///
/// Returns an error if the geocoding or weather API calls fail.
#[instrument(fields(location = %request.location), skip(client))]
pub async fn get_weather_with_client_and_urls(
    client: &ClientWithMiddleware,
    request: WeatherRequest,
    geocoding_base_url: &str,
    weather_base_url: &str,
) -> Result<WeatherResponse> {
    let geocoding_url = format!(
        "{}/v1/search?name={}&count=1&language=en&format=json",
        geocoding_base_url.trim_end_matches('/'),
        urlencoding::encode(&request.location)
    );

    let geo_response: GeocodingResponse = client
        .get(&geocoding_url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .context("Failed to fetch geocoding data")?
        .error_for_status()
        .context("Geocoding API returned error")?
        .json()
        .await
        .context("Failed to parse geocoding response")?;

    let geo_result = geo_response
        .results
        .and_then(|mut r| r.pop())
        .context("Location not found")?;

    debug!(
        name = %geo_result.name,
        lat = %geo_result.latitude,
        lon = %geo_result.longitude,
        country = ?geo_result.country_code,
        "Geocoding result"
    );

    let weather_url = format!(
        "{}/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code,wind_speed_10m",
        weather_base_url.trim_end_matches('/'),
        geo_result.latitude, geo_result.longitude
    );

    let weather_response: OpenMeteoResponse = client
        .get(&weather_url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .context("Failed to fetch weather data")?
        .error_for_status()
        .context("Weather API returned error")?
        .json()
        .await
        .context("Failed to parse weather response")?;

    let temperature_unit = geo_result
        .country_code
        .as_deref()
        .map_or(TemperatureUnit::C, TemperatureUnit::from_country_code);

    let temperature =
        temperature_unit.convert_from_celsius(weather_response.current.temperature_2m);

    debug!(
        temp_celsius = %weather_response.current.temperature_2m,
        temp_converted = %temperature,
        unit = ?temperature_unit,
        "Temperature converted"
    );

    Ok(WeatherResponse {
        location: geo_result.name,
        temperature,
        temperature_unit,
        weather_code: weather_response.current.weather_code,
        wind_speed: weather_response.current.wind_speed_10m,
    })
}

/// Internal function that allows injecting an HTTP client for testing
///
/// # Errors
///
/// Returns an error if the geocoding or weather API calls fail.
#[instrument(fields(location = %request.location), skip(client))]
pub async fn get_weather_with_client(
    client: &ClientWithMiddleware,
    request: WeatherRequest,
) -> Result<WeatherResponse> {
    get_weather_with_client_and_urls(
        client,
        request,
        "https://geocoding-api.open-meteo.com",
        "https://api.open-meteo.com",
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_weather_success_celsius() {
        let mock_server = MockServer::start().await;

        // Mock geocoding response for Sydney, Australia
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{
                    "name": "Sydney",
                    "latitude": -33.8688,
                    "longitude": 151.2093,
                    "country_code": "AU"
                }]
            })))
            .mount(&mock_server)
            .await;

        // Mock weather response
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "current": {
                    "temperature_2m": 25.0,
                    "weather_code": 1,
                    "wind_speed_10m": 15.0
                }
            })))
            .mount(&mock_server)
            .await;

        let reqwest_client = reqwest::Client::new();
        let client = reqwest_middleware::ClientBuilder::new(reqwest_client).build();
        let request = WeatherRequest {
            location: "Sydney".to_string(),
        };

        let result = get_weather_with_client_and_urls(
            &client,
            request,
            &mock_server.uri(),
            &mock_server.uri(),
        )
        .await
        .unwrap();

        assert_eq!(result.location, "Sydney");
        assert!((result.temperature - 25.0).abs() < f64::EPSILON); // Should be Celsius for AU
        assert_eq!(result.temperature_unit, TemperatureUnit::C);
        assert_eq!(result.weather_code, 1);
        assert!((result.wind_speed - 15.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_get_weather_success_fahrenheit() {
        let mock_server = MockServer::start().await;

        // Mock geocoding response for New York, USA
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{
                    "name": "New York",
                    "latitude": 40.7128,
                    "longitude": -74.0060,
                    "country_code": "US"
                }]
            })))
            .mount(&mock_server)
            .await;

        // Mock weather response
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "current": {
                    "temperature_2m": 20.0,
                    "weather_code": 2,
                    "wind_speed_10m": 10.0
                }
            })))
            .mount(&mock_server)
            .await;

        let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build();
        let request = WeatherRequest {
            location: "New York".to_string(),
        };

        let result = get_weather_with_client_and_urls(
            &client,
            request,
            &mock_server.uri(),
            &mock_server.uri(),
        )
        .await
        .unwrap();

        assert_eq!(result.location, "New York");
        assert!((result.temperature - 68.0).abs() < f64::EPSILON); // 20°C = 68°F
        assert_eq!(result.temperature_unit, TemperatureUnit::F);
        assert_eq!(result.weather_code, 2);
        assert!((result.wind_speed - 10.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_get_weather_location_not_found() {
        let mock_server = MockServer::start().await;

        // Mock geocoding response with no results
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": null
            })))
            .mount(&mock_server)
            .await;

        let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build();
        let request = WeatherRequest {
            location: "NonExistentLocation".to_string(),
        };

        let result = get_weather_with_client_and_urls(
            &client,
            request,
            &mock_server.uri(),
            &mock_server.uri(),
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Location not found"));
    }

    #[tokio::test]
    async fn test_get_weather_geocoding_api_error() {
        let mock_server = MockServer::start().await;

        // Mock geocoding API error
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build();
        let request = WeatherRequest {
            location: "Sydney".to_string(),
        };

        let result = get_weather_with_client_and_urls(
            &client,
            request,
            &mock_server.uri(),
            &mock_server.uri(),
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Geocoding API returned error"));
    }

    #[tokio::test]
    async fn test_get_weather_weather_api_error() {
        let mock_server = MockServer::start().await;

        // Mock successful geocoding
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{
                    "name": "Sydney",
                    "latitude": -33.8688,
                    "longitude": 151.2093,
                    "country_code": "AU"
                }]
            })))
            .mount(&mock_server)
            .await;

        // Mock weather API error
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build();
        let request = WeatherRequest {
            location: "Sydney".to_string(),
        };

        let result = get_weather_with_client_and_urls(
            &client,
            request,
            &mock_server.uri(),
            &mock_server.uri(),
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Weather API returned error"));
    }

    #[tokio::test]
    async fn test_get_weather_invalid_json_response() {
        let mock_server = MockServer::start().await;

        // Mock geocoding with invalid JSON
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
            .mount(&mock_server)
            .await;

        let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build();
        let request = WeatherRequest {
            location: "Sydney".to_string(),
        };

        let result = get_weather_with_client_and_urls(
            &client,
            request,
            &mock_server.uri(),
            &mock_server.uri(),
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse geocoding response"));
    }

    #[tokio::test]
    async fn test_get_weather_no_country_code_defaults_to_celsius() {
        let mock_server = MockServer::start().await;

        // Mock geocoding response without country code
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{
                    "name": "Unknown Location",
                    "latitude": 0.0,
                    "longitude": 0.0,
                    "country_code": null
                }]
            })))
            .mount(&mock_server)
            .await;

        // Mock weather response
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "current": {
                    "temperature_2m": 15.0,
                    "weather_code": 0,
                    "wind_speed_10m": 5.0
                }
            })))
            .mount(&mock_server)
            .await;

        let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build();
        let request = WeatherRequest {
            location: "Unknown Location".to_string(),
        };

        let result = get_weather_with_client_and_urls(
            &client,
            request,
            &mock_server.uri(),
            &mock_server.uri(),
        )
        .await
        .unwrap();

        assert_eq!(result.location, "Unknown Location");
        assert!((result.temperature - 15.0).abs() < f64::EPSILON); // Should default to Celsius
        assert_eq!(result.temperature_unit, TemperatureUnit::C);
    }
}
