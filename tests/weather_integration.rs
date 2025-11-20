use anyhow::Result;
use aws_lambda_mcp::models::weather::{TemperatureUnit, WeatherRequest};
use aws_lambda_mcp::tools::get_weather;

#[tokio::test]
async fn test_get_weather_integration_sydney() -> Result<()> {
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
async fn test_get_weather_integration_new_york() -> Result<()> {
    // This test requires network access to Open-Meteo APIs
    // Skip in CI or when network is unavailable
    if std::env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return Ok(());
    }

    let request = WeatherRequest {
        location: "New York".to_string(),
    };

    let response = get_weather(request).await?;
    assert_eq!(response.location, "New York");
    assert!(response.temperature.is_finite());
    assert!(matches!(response.temperature_unit, TemperatureUnit::F));
    assert!(response.weather_code >= 0);
    assert!(response.wind_speed >= 0.0);

    Ok(())
}

#[tokio::test]
async fn test_get_weather_integration_london() -> Result<()> {
    // This test requires network access to Open-Meteo APIs
    // Skip in CI or when network is unavailable
    if std::env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return Ok(());
    }

    let request = WeatherRequest {
        location: "London".to_string(),
    };

    let response = get_weather(request).await?;
    assert_eq!(response.location, "London");
    assert!(response.temperature.is_finite());
    assert!(matches!(response.temperature_unit, TemperatureUnit::C));
    assert!(response.weather_code >= 0);
    assert!(response.wind_speed >= 0.0);

    Ok(())
}

#[tokio::test]
async fn test_get_weather_integration_tokyo() -> Result<()> {
    // This test requires network access to Open-Meteo APIs
    // Skip in CI or when network is unavailable
    if std::env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return Ok(());
    }

    let request = WeatherRequest {
        location: "Tokyo".to_string(),
    };

    let response = get_weather(request).await?;
    assert_eq!(response.location, "Tokyo");
    assert!(response.temperature.is_finite());
    assert!(matches!(response.temperature_unit, TemperatureUnit::C));
    assert!(response.weather_code >= 0);
    assert!(response.wind_speed >= 0.0);

    Ok(())
}

#[tokio::test]
async fn test_get_weather_integration_address() -> Result<()> {
    // This test requires network access to Open-Meteo APIs
    // Skip in CI or when network is unavailable
    if std::env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return Ok(());
    }

    let request = WeatherRequest {
        location: "1600 Pennsylvania Avenue NW, Washington, DC".to_string(),
    };

    let response = get_weather(request).await?;
    // The location name might be normalized by the geocoding API
    assert!(response.location.contains("Washington") || response.location.contains("Pennsylvania"));
    assert!(response.temperature.is_finite());
    assert!(matches!(response.temperature_unit, TemperatureUnit::F)); // USA should use Fahrenheit
    assert!(response.weather_code >= 0);
    assert!(response.wind_speed >= 0.0);

    Ok(())
}

#[tokio::test]
async fn test_get_weather_integration_edge_cases() -> Result<()> {
    // This test requires network access to Open-Meteo APIs
    // Skip in CI or when network is unavailable
    if std::env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return Ok(());
    }

    // Test with special characters and encoding
    let locations = vec![
        "Paris, France",  // Comma in location
        "São Paulo",      // Non-ASCII characters
        "Moscow, Russia", // Different timezone
    ];

    for location in locations {
        let request = WeatherRequest {
            location: location.to_string(),
        };

        let response = get_weather(request).await?;
        assert!(response.temperature.is_finite());
        assert!(response.weather_code >= 0);
        assert!(response.wind_speed >= 0.0);
    }

    Ok(())
}
