use anyhow::{Context, Result};
use crate::http::HTTP_CLIENT;
use crate::models::{WeatherRequest, WeatherResponse};
use crate::models::open_meteo::OpenMeteoResponse;

/// Fetches weather data from the Open-Meteo API.
///
/// # Errors
///
/// This function will return an error if:
/// - The HTTP request to the Open-Meteo API fails.
/// - The response from the Open-Meteo API cannot be parsed.
pub async fn get_weather(request: WeatherRequest) -> Result<WeatherResponse> {
    let daily_params = request.daily.join(",");
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily={}&timezone={}",
        request.latitude, request.longitude, daily_params, request.timezone
    );

    let client = &HTTP_CLIENT;

    let response: reqwest::Response = client
        .get(&url)
        .send()
        .await
        .context("Failed to send request to OpenMeteo")?;

    let response: OpenMeteoResponse = response
        .json()
        .await
        .context("Failed to parse response from OpenMeteo")?;

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
