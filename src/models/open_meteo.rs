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
