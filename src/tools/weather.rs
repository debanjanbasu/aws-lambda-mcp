use anyhow::{Context, Result};
use rmcp::tool;
use std::time::Duration;

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
        .json::<GeocodingResponse>()
        .await
        .context("Failed to parse geocoding response")?;

    let geo_result = geo_response
        .results
        .and_then(|mut r: Vec<_>| r.pop())
        .context("Location not found")?;



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
        .json::<OpenMeteoResponse>()
        .await
        .context("Failed to parse weather response")?;

    let temperature_unit = geo_result
        .country_code
        .as_deref()
        .map_or(TemperatureUnit::C, TemperatureUnit::from_country_code);

    let temperature =
        temperature_unit.convert_from_celsius(weather_response.current.temperature_2m);



    Ok(WeatherResponse {
        location: geo_result.name,
        temperature,
        temperature_unit,
        weather_code: weather_response.current.weather_code,
        wind_speed: weather_response.current.wind_speed_10m,
    })
}
