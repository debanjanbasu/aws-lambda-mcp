use anyhow::{Context, Result};
use serde::Deserialize;
use std::time::Duration;
use tracing::{debug, instrument};

use crate::http::HTTP_CLIENT;
use crate::models::{TemperatureUnit, WeatherRequest, WeatherResponse};

#[derive(Debug, Deserialize)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}

#[derive(Debug, Deserialize)]
struct GeocodingResult {
    name: String,
    latitude: f64,
    longitude: f64,
    country_code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current: CurrentWeather,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    temperature_2m: f64,
    weather_code: i32,
    wind_speed_10m: f64,
}

/// Get current weather information for a specified location.
///
/// Returns temperature (automatically converted to Celsius or Fahrenheit based on the country),
/// WMO weather code, and wind speed in km/h. Supports city names, addresses, or place names worldwide.
///
/// # Errors
///
/// Returns an error if the geocoding or weather API calls fail.
#[rmcp::tool(
    description = "Get current weather information for a specified location. Returns temperature (automatically converted to Celsius or Fahrenheit based on the country), WMO weather code, and wind speed in km/h. Supports city names, addresses, or place names worldwide."
)]
#[instrument(fields(location = %request.location))]
pub async fn get_weather(request: WeatherRequest) -> Result<WeatherResponse> {
    let geocoding_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        urlencoding::encode(&request.location)
    );

    let geo_response: GeocodingResponse = HTTP_CLIENT
        .get(&geocoding_url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .context("Failed to fetch geocoding data")?
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
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code,wind_speed_10m",
        geo_result.latitude, geo_result.longitude
    );

    let weather_response: OpenMeteoResponse = HTTP_CLIENT
        .get(&weather_url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .context("Failed to fetch weather data")?
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
