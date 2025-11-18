use anyhow::Result;
use aws_lambda_mcp::models::weather::{TemperatureUnit, WeatherRequest};
use aws_lambda_mcp::tools::get_weather;

#[tokio::test]
async fn test_get_weather_integration() -> Result<()> {
    // This test requires network access to Open-Meteo APIs
    // Skip in CI or when network is unavailable
    if std::env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return Ok(());
    }

    let request = WeatherRequest {
        location: "Sydney".to_string(),
    };

    let response = get_weather(request).await?;
    assert_eq!(response.location, "Sydney");
    assert!(response.temperature.is_finite());
    assert!(matches!(response.temperature_unit, TemperatureUnit::C));
    assert!(response.weather_code >= 0);
    assert!(response.wind_speed >= 0.0);

    Ok(())
}

#[tokio::test]
async fn test_get_weather_integration_us_location() -> Result<()> {
    // Test location in US to verify Fahrenheit conversion
    if std::env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return Ok(());
    }

    let request = WeatherRequest {
        location: "New York".to_string(),
    };

    let response = get_weather(request).await?;
    assert!(response.location.contains("New York") || response.location.contains("New York City"));
    assert!(response.temperature.is_finite());
    assert!(matches!(response.temperature_unit, TemperatureUnit::F));
    assert!(response.weather_code >= 0);
    assert!(response.wind_speed >= 0.0);

    Ok(())
}

#[tokio::test]
async fn test_get_weather_integration_european_location() -> Result<()> {
    // Test location in Europe to verify Celsius
    if std::env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return Ok(());
    }

    let request = WeatherRequest {
        location: "London".to_string(),
    };

    let response = get_weather(request).await?;
    assert!(response.location.contains("London"));
    assert!(response.temperature.is_finite());
    assert!(matches!(response.temperature_unit, TemperatureUnit::C));
    assert!(response.weather_code >= 0);
    assert!(response.wind_speed >= 0.0);

    Ok(())
}

#[tokio::test]
async fn test_get_weather_integration_invalid_location() -> Result<()> {
    // Test invalid location handling
    if std::env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return Ok(());
    }

    let request = WeatherRequest {
        location: "NonExistentCity12345".to_string(),
    };

    let result = get_weather(request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Location not found"));

    Ok(())
}
