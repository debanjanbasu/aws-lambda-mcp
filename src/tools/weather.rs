use crate::http::HTTP_CLIENT;
use crate::models::open_meteo::OpenMeteoResponse;
use crate::models::{WeatherRequest, WeatherResponse};
use anyhow::{Context, Result};
use lambda_runtime::tracing::info;

/// Fetches weather data from the Open-Meteo API.
///
/// # Errors
///
/// This function will return an error if:
/// - The HTTP request to geocode the location fails.
/// - The HTTP request to the Open-Meteo API fails.
/// - The response from either API cannot be parsed.
pub async fn get_weather(request: WeatherRequest) -> Result<WeatherResponse> {
    info!("Starting weather request for location: {}", request.location);
    
    // Use the global HTTP client
    let client = &HTTP_CLIENT;

    // First, geocode the location to get coordinates
    let geocode_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        urlencoding::encode(&request.location)
    );
    
    info!("Making geocoding request to: {}", geocode_url);

    let geocode_response: serde_json::Value = client
        .get(&geocode_url)
        .send()
        .await
        .context("Failed to send geocoding request to Open-Meteo geocoding API")?
        .json()
        .await
        .context("Failed to parse geocoding response from Open-Meteo")?;

    info!("Received geocoding response: {:?}", geocode_response);

    // Extract coordinates from geocoding response
    let (latitude, longitude, timezone) = extract_coordinates_from_geocode(&geocode_response)
        .context("Failed to extract coordinates from geocoding response")?;

    info!("Extracted coordinates: lat={}, lng={}, timezone={}", latitude, longitude, timezone);

    // Use sensible defaults for daily weather parameters
    let daily_params = ["weather_code", "temperature_2m_max", "temperature_2m_min"];
    let daily_params_str = daily_params.join(",");

    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={latitude}&longitude={longitude}&daily={daily_params_str}&timezone={timezone}"
    );
    
    info!("Making weather forecast request to: {}", weather_url);

    let response: reqwest::Response = client
        .get(&weather_url)
        .send()
        .await
        .context("Failed to send weather forecast request to Open-Meteo API")?;

    info!("Received weather forecast response with status: {}", response.status());

    let response: OpenMeteoResponse = response
        .json()
        .await
        .context("Failed to parse weather forecast response from Open-Meteo")?;

    info!("Parsed weather forecast response successfully");

    Ok(WeatherResponse {
        latitude: response.latitude,
        longitude: response.longitude,
        generationtime_ms: response.generationtime_ms,
        utc_offset_seconds: response.utc_offset_seconds,
        timezone: response.timezone,
        timezone_abbreviation: response.timezone_abbreviation,
        elevation: response.elevation,
        daily_units: response.daily_units.into(),
        daily: response.daily.into(),
    })
}

/// Extracts coordinates and timezone from geocoding API response
fn extract_coordinates_from_geocode(
    geocode_response: &serde_json::Value,
) -> Result<(f64, f64, String)> {
    let results = geocode_response
        .get("results")
        .and_then(serde_json::Value::as_array)
        .context("No results found in geocoding response")?;

    if results.is_empty() {
        anyhow::bail!("No locations found for the provided query");
    }

    let first_result = &results[0];
    let latitude = first_result
        .get("latitude")
        .and_then(serde_json::Value::as_f64)
        .context("Failed to extract latitude")?;

    let longitude = first_result
        .get("longitude")
        .and_then(serde_json::Value::as_f64)
        .context("Failed to extract longitude")?;

    let timezone = first_result
        .get("timezone")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("auto")
        .to_string();

    Ok((latitude, longitude, timezone))
}

impl From<crate::models::open_meteo::DailyUnits> for crate::models::weather::DailyUnits {
    fn from(units: crate::models::open_meteo::DailyUnits) -> Self {
        Self {
            time: units.time,
            weather_code: units.weather_code,
            temperature_2m_max: units.temperature_2m_max,
            temperature_2m_min: units.temperature_2m_min,
        }
    }
}

impl From<crate::models::open_meteo::Daily> for crate::models::weather::Daily {
    fn from(daily: crate::models::open_meteo::Daily) -> Self {
        Self {
            time: daily.time,
            weather_code: daily.weather_code,
            temperature_2m_max: daily.temperature_2m_max,
            temperature_2m_min: daily.temperature_2m_min,
        }
    }
}
