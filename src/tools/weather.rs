use crate::http::HTTP_CLIENT;
use crate::models::error::AppError;
use crate::models::open_meteo::OpenMeteoResponse;
use crate::models::{WeatherRequest, WeatherResponse};
use anyhow::Result;
use lambda_runtime::tracing::info;

/// Default daily weather parameters for Open-Meteo API requests
const DEFAULT_DAILY_PARAMS: [&str; 3] =
    ["weather_code", "temperature_2m_max", "temperature_2m_min"];

/// Fetches weather data from the Open-Meteo API.
///
/// This function simplifies weather requests by:
/// 1. Converting location names to coordinates via geocoding
/// 2. Using sensible defaults for weather parameters
/// 3. Automatically handling timezone detection
///
/// # Errors
///
/// This function will return an error if:
/// - The HTTP request to geocode the location fails
/// - No locations are found for the provided query
/// - Failed to extract coordinates from geocoding response
/// - The HTTP request to the Open-Meteo API fails
/// - The response from either API cannot be parsed
pub async fn get_weather(request: WeatherRequest) -> Result<WeatherResponse, AppError> {
    info!(
        "Starting weather request for location: {}",
        request.location
    );

    // Get coordinates for the location
    let (latitude, longitude, timezone) = geocode_location(&request.location).await?;

    // Fetch weather data
    let weather_data = fetch_weather_data(latitude, longitude, &timezone).await?;

    info!("Successfully fetched weather data");
    Ok(weather_data)
}

/// Geocodes a location name to coordinates
async fn geocode_location(location: &str) -> Result<(f64, f64, String), AppError> {
    let encoded_location = urlencoding::encode(location);
    let geocode_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={encoded_location}&count=1&language=en&format=json"
    );

    info!("Geocoding location: {}", location);
    info!("Making geocoding request to: {}", geocode_url);

    let client = &HTTP_CLIENT;
    let response: serde_json::Value = client
        .get(&geocode_url)
        .send()
        .await
        .map_err(|e| AppError::GeocodingError(format!("Failed to send geocoding request: {e}")))?
        .json()
        .await
        .map_err(|e| {
            AppError::GeocodingError(format!("Failed to parse geocoding response: {e}"))
        })?;

    info!("Received geocoding response");

    extract_coordinates_from_geocode(&response)
}

/// Fetches weather data for the given coordinates
async fn fetch_weather_data(
    latitude: f64,
    longitude: f64,
    timezone: &str,
) -> Result<WeatherResponse, AppError> {
    let daily_params_str = DEFAULT_DAILY_PARAMS.join(",");
    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={latitude}&longitude={longitude}&daily={daily_params_str}&timezone={timezone}"
    );

    info!(
        "Fetching weather data for coordinates: {}, {}",
        latitude, longitude
    );
    info!("Making weather forecast request to: {}", weather_url);

    let client = &HTTP_CLIENT;
    let response = client.get(&weather_url).send().await.map_err(|e| {
        AppError::WeatherApiError(format!("Failed to send weather forecast request: {e}"))
    })?;

    info!(
        "Received weather forecast response with status: {}",
        response.status()
    );

    // Check if the response is successful
    if !response.status().is_success() {
        return Err(AppError::WeatherApiError(format!(
            "Weather API returned non-success status: {}",
            response.status()
        )));
    }

    let open_meteo_response: OpenMeteoResponse = response.json().await.map_err(|e| {
        AppError::WeatherApiError(format!("Failed to parse weather forecast response: {e}"))
    })?;

    info!("Parsed weather forecast response successfully");

    Ok(WeatherResponse {
        latitude: open_meteo_response.latitude,
        longitude: open_meteo_response.longitude,
        generationtime_ms: open_meteo_response.generationtime_ms,
        utc_offset_seconds: open_meteo_response.utc_offset_seconds,
        timezone: open_meteo_response.timezone,
        timezone_abbreviation: open_meteo_response.timezone_abbreviation,
        elevation: open_meteo_response.elevation,
        daily_units: open_meteo_response.daily_units.into(),
        daily: open_meteo_response.daily.into(),
    })
}

/// Extracts coordinates and timezone from geocoding API response
fn extract_coordinates_from_geocode(
    geocode_response: &serde_json::Value,
) -> Result<(f64, f64, String), AppError> {
    let results = geocode_response
        .get("results")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            AppError::GeocodingError("No results found in geocoding response".to_string())
        })?;

    if results.is_empty() {
        return Err(AppError::GeocodingError(
            "No locations found for the provided query".to_string(),
        ));
    }

    let first_result = &results[0];
    let latitude = first_result
        .get("latitude")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| AppError::GeocodingError("Failed to extract latitude".to_string()))?;

    let longitude = first_result
        .get("longitude")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| AppError::GeocodingError("Failed to extract longitude".to_string()))?;

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
