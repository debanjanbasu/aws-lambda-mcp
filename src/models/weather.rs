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
    fn test_temperature_unit_default() {
        assert_eq!(TemperatureUnit::default(), TemperatureUnit::C);
    }

    #[test]
    fn test_temperature_unit_serialization() {
        let unit = TemperatureUnit::F;
        let serialized = serde_json::to_string(&unit).unwrap();
        assert_eq!(serialized, "\"F\"");

        let deserialized: TemperatureUnit = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, TemperatureUnit::F);
    }
}

#[cfg(test)]
mod request_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_weather_request_deserialization() {
        let json = r#"{"location": "Sydney, Australia"}"#;
        let request: WeatherRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.location, "Sydney, Australia");
    }

    #[test]
    fn test_weather_request_missing_location() {
        let json = r#"{"other_field": "value"}"#;
        let result: Result<WeatherRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod response_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_weather_response_serialization() {
        let response = WeatherResponse {
            location: "Sydney".to_string(),
            temperature: 25.5,
            temperature_unit: TemperatureUnit::C,
            weather_code: 1,
            wind_speed: 15.2,
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: WeatherResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.location, "Sydney");
        assert!((deserialized.temperature - 25.5).abs() < f64::EPSILON);
        assert_eq!(deserialized.temperature_unit, TemperatureUnit::C);
        assert_eq!(deserialized.weather_code, 1);
        assert!((deserialized.wind_speed - 15.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_weather_response_json_schema() {
        let schema = schemars::schema_for!(WeatherResponse);
        let schema_json = serde_json::to_string_pretty(&schema).unwrap();

        // Basic checks that the schema contains expected fields
        assert!(schema_json.contains("location"));
        assert!(schema_json.contains("temperature"));
        assert!(schema_json.contains("temperature_unit"));
        assert!(schema_json.contains("weather_code"));
        assert!(schema_json.contains("wind_speed"));
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
