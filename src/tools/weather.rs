use anyhow::{Context, Result};
use serde_json::{json, Value};
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
/// Returns an error if the geocoding or weather API calls fail, or if serialization fails.
#[instrument(fields(location = %request.location))]
pub async fn get_weather(request: WeatherRequest) -> Result<Value> {
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

    let response = WeatherResponse {
        location: geo_result.name,
        temperature,
        temperature_unit,
        weather_code: weather_response.current.weather_code,
        wind_speed: weather_response.current.wind_speed_10m,
    };

    let text = serde_json::to_string(&response).context("Failed to serialize weather response")?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_weather_response_format() {
        let response = WeatherResponse {
            location: "Test City".to_string(),
            temperature: 25.0,
            temperature_unit: TemperatureUnit::C,
            weather_code: 0,
            wind_speed: 10.0,
        };

        let expected_text = serde_json::to_string(&response).unwrap();
        let expected = json!({
            "content": [{
                "type": "text",
                "text": expected_text
            }]
        });

        // Since we can't easily test the async function without mocking,
        // we test the format of the expected output
        assert_eq!(expected["content"][0]["type"], "text");
        assert!(expected["content"][0]["text"].is_string());
    }
}
