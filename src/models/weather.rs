use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Temperature unit for weather measurements
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_country_code() {
        assert_eq!(TemperatureUnit::from_country_code("US"), TemperatureUnit::F);
        assert_eq!(TemperatureUnit::from_country_code("us"), TemperatureUnit::F);
        assert_eq!(TemperatureUnit::from_country_code("LR"), TemperatureUnit::F);
        assert_eq!(TemperatureUnit::from_country_code("MM"), TemperatureUnit::F);
        assert_eq!(TemperatureUnit::from_country_code("AU"), TemperatureUnit::C);
        assert_eq!(TemperatureUnit::from_country_code(""), TemperatureUnit::C);
    }

    #[test]
    fn test_convert_from_celsius() {
        assert!((TemperatureUnit::C.convert_from_celsius(20.0) - 20.0).abs() < f64::EPSILON);
        assert!((TemperatureUnit::F.convert_from_celsius(20.0) - 68.0).abs() < f64::EPSILON);
        assert!((TemperatureUnit::F.convert_from_celsius(0.0) - 32.0).abs() < f64::EPSILON);
        assert!((TemperatureUnit::F.convert_from_celsius(-40.0) - (-40.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_temperature_unit_serialization() {
        let c: TemperatureUnit = serde_json::from_str("\"C\"").unwrap();
        assert_eq!(c, TemperatureUnit::C);

        let f: TemperatureUnit = serde_json::from_str("\"F\"").unwrap();
        assert_eq!(f, TemperatureUnit::F);

        assert_eq!(serde_json::to_string(&TemperatureUnit::C).unwrap(), "\"C\"");
        assert_eq!(serde_json::to_string(&TemperatureUnit::F).unwrap(), "\"F\"");
    }

    #[test]
    fn test_weather_request_serialization() {
        let request = WeatherRequest {
            location: "Sydney".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: WeatherRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.location, "Sydney");
    }

    #[test]
    fn test_weather_response_serialization() {
        let response = WeatherResponse {
            location: "Sydney".to_string(),
            temperature: 25.0,
            temperature_unit: TemperatureUnit::C,
            weather_code: 0,
            wind_speed: 10.5,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: WeatherResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.location, "Sydney");
        assert!((deserialized.temperature - 25.0).abs() < f64::EPSILON);
        assert_eq!(deserialized.temperature_unit, TemperatureUnit::C);
        assert_eq!(deserialized.weather_code, 0);
        assert!((deserialized.wind_speed - 10.5).abs() < f64::EPSILON);
    }
}

/// Request for weather information
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct WeatherRequest {
    #[schemars(description = "Location name (city, address, or place)")]
    pub location: String,
}

/// Response containing weather information
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
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
