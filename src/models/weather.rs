use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Temperature unit for weather measurements
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum TemperatureUnit {
    #[default]
    C,
    F,
}

impl TemperatureUnit {
    /// Determines temperature unit based on country code (case-insensitive)
    ///
    /// Countries using Fahrenheit: US, Liberia, Myanmar
    #[must_use]
    pub fn from_country_code(country_code: &str) -> Self {
        matches!(country_code, "US" | "us" | "LR" | "lr" | "MM" | "mm")
            .then_some(Self::F)
            .unwrap_or(Self::C)
    }

    /// Converts a temperature value to this unit from Celsius
    #[inline]
    #[must_use]
    pub const fn convert_from_celsius(self, celsius: f64) -> f64 {
        match self {
            Self::C => celsius,
            Self::F => celsius * 1.8 + 32.0,
        }
    }
}

/// Request for weather information
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct WeatherRequest {
    #[schemars(description = "Location name (city, address, or place)")]
    pub location: String,
}

/// Response containing weather information
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct WeatherResponse {
    #[schemars(description = "Location name")]
    pub location: String,
    #[schemars(description = "Temperature value")]
    pub temperature: f64,
    #[schemars(description = "The unit of temperature (Celsius or Fahrenheit)")]
    pub temperature_unit: TemperatureUnit,
    #[schemars(description = "WMO weather code")]
    pub weather_code: i32,
    #[schemars(description = "Wind speed in km/h")]
    pub wind_speed: f64,
}
