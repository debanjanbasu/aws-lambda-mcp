use serde::Deserialize;

// Response from geocoding API
#[derive(Debug, Deserialize)]
pub struct GeocodingResponse {
    pub results: Option<Vec<GeocodingResult>>,
}

// Geocoding result with location coordinates
#[derive(Debug, Deserialize)]
pub struct GeocodingResult {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country_code: Option<String>,
}

// Response from OpenMeteo weather API
#[derive(Debug, Deserialize)]
pub struct OpenMeteoResponse {
    pub current: CurrentWeather,
}

// Current weather data
#[derive(Debug, Deserialize)]
pub struct CurrentWeather {
    pub temperature_2m: f64,
    pub weather_code: i32,
    pub wind_speed_10m: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geocoding_response_deserialization() {
        let json = r#"{
            "results": [
                {
                    "name": "Sydney",
                    "latitude": -33.8688,
                    "longitude": 151.2093,
                    "country_code": "AU"
                }
            ]
        }"#;

        let response: GeocodingResponse = serde_json::from_str(json).unwrap();
        let result = response.results.unwrap().into_iter().next().unwrap();
        assert_eq!(result.name, "Sydney");
        assert!((result.latitude - (-33.8688)).abs() < f64::EPSILON);
        assert!((result.longitude - 151.2093).abs() < f64::EPSILON);
        assert_eq!(result.country_code, Some("AU".to_string()));
    }

    #[test]
    fn test_geocoding_response_no_results() {
        let json = r#"{"results": null}"#;

        let response: GeocodingResponse = serde_json::from_str(json).unwrap();
        assert!(response.results.is_none());
    }

    #[test]
    fn test_open_meteo_response_deserialization() {
        let json = r#"{
            "current": {
                "temperature_2m": 25.0,
                "weather_code": 0,
                "wind_speed_10m": 10.5
            }
        }"#;

        let response: OpenMeteoResponse = serde_json::from_str(json).unwrap();
        assert!((response.current.temperature_2m - 25.0).abs() < f64::EPSILON);
        assert_eq!(response.current.weather_code, 0);
        assert!((response.current.wind_speed_10m - 10.5).abs() < f64::EPSILON);
    }
}
