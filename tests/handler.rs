// Handler tests
#![allow(clippy::unwrap_used)]

use aws_lambda_mcp::handler::route_tool;
use aws_lambda_mcp::tools::weather::{get_weather_with_client, HttpClient, MockClient};
use serde_json::json;
use std::fs;
use std::process::Command;

#[tokio::test]
async fn test_route_tool_unknown() {
    let event_payload = json!({"name": "unknown_tool"});
    let result = route_tool("unknown_tool", event_payload).await;
    assert!(result.is_err(), "Expected error for unknown tool");

    if let Err(err) = result {
        assert_eq!(err.error_type, "UnknownTool");
        assert!(err.error_message.contains("Unknown tool: unknown_tool"));
    }
}

#[tokio::test]
async fn test_weather_tool_successful_request_with_mocked_api() {
    // Create mock HTTP client
    let mut mock_client = MockClient::new();

    // Mock geocoding response for "New York"
    mock_client.mock_response("geocoding-api.open-meteo.com", json!({
        "results": [{
            "latitude": 40.7128,
            "longitude": -74.0060,
            "timezone": "America/New_York"
        }]
    }));

    // Mock weather API response
    mock_client.mock_response("api.open-meteo.com", json!({
        "latitude": 40.7128,
        "longitude": -74.0060,
        "generationtime_ms": 0.5,
        "utc_offset_seconds": -18000,
        "timezone": "America/New_York",
        "timezone_abbreviation": "EST",
        "elevation": 10.0,
        "daily_units": {
            "time": "iso8601",
            "weather_code": "wmo code",
            "temperature_2m_max": "°C",
            "temperature_2m_min": "°C"
        },
        "daily": {
            "time": ["2024-01-01"],
            "weather_code": [1],
            "temperature_2m_max": [5.0],
            "temperature_2m_min": [-2.0]
        }
    }));

    let http_client = HttpClient::Mock(mock_client);

    // Test the weather tool with mocked client
    let request = aws_lambda_mcp::models::WeatherRequest {
        location: "New York".to_string(),
    };

    let result = get_weather_with_client(&http_client, request).await;
    assert!(result.is_ok(), "Weather request should succeed with mocked responses");

    let response = result.unwrap();
    assert_eq!(response.latitude, 40.7128);
    assert_eq!(response.longitude, -74.0060);
    assert_eq!(response.timezone, "America/New_York");
}

#[tokio::test]
async fn test_weather_invalid_arguments() {
    // Simulate MCP request structure with invalid arguments for get_weather
    let mcp_payload = json!({
        "method": "tools/call",
        "params": {
            "arguments": {
                "invalid_field": "New York"
            }
        }
    });

    let result = route_tool("get_weather", mcp_payload).await;
    assert!(result.is_err(), "Expected error for invalid arguments");

    if let Err(err) = result {
        assert_eq!(err.error_type, "InvalidInput");
    }
}

#[tokio::test]
async fn test_personalized_greeting_with_user_name() {
    // Simulate MCP request structure with user information for get_personalized_greeting
    let mcp_payload = json!({
        "method": "tools/call",
        "params": {
            "arguments": {
                "user_name": "John",
                "user_id": "john@example.com"
            }
        }
    });

    let result = route_tool("get_personalized_greeting", mcp_payload).await;
    assert_successful_greeting(result, "John");
}

#[tokio::test]
async fn test_personalized_greeting_with_user_id_only() {
    // Simulate MCP request structure with only user ID for get_personalized_greeting
    let mcp_payload = json!({
        "method": "tools/call",
        "params": {
            "arguments": {
                "user_id": "jane.doe@example.com"
            }
        }
    });

    let result = route_tool("get_personalized_greeting", mcp_payload).await;
    assert_successful_greeting(result, "jane.doe");
}

#[tokio::test]
async fn test_personalized_greeting_without_user_info() {
    // Simulate MCP request structure without user information for get_personalized_greeting
    let mcp_payload = json!({
        "method": "tools/call",
        "params": {
            "arguments": {}
        }
    });

    let result = route_tool("get_personalized_greeting", mcp_payload).await;
    assert_successful_greeting(result, "there");
}

#[tokio::test]
async fn test_personalized_greeting_invalid_arguments() {
    // Simulate MCP request structure with invalid arguments
    let mcp_payload = json!({
        "method": "tools/call",
        "params": {
            "arguments": {
                "invalid_field": "some_value"
            }
        }
    });

    let result = route_tool("get_personalized_greeting", mcp_payload).await;
    // Even with invalid fields, this should succeed with default greeting
    assert_successful_greeting(result, "there");
}

#[tokio::test]
async fn test_weather_geocoding_failure() {
    let mut mock_client = MockClient::new();

    // Mock geocoding failure (empty results)
    mock_client.mock_response("geocoding-api.open-meteo.com", json!({"results": []}));

    let http_client = HttpClient::Mock(mock_client);

    let request = aws_lambda_mcp::models::WeatherRequest {
        location: "NonExistentCity".to_string(),
    };

    let result = get_weather_with_client(&http_client, request).await;
    assert!(result.is_err(), "Should fail when no geocoding results found");

    if let Err(err) = result {
        match err {
            aws_lambda_mcp::models::AppError::GeocodingError(msg) => {
                assert!(msg.contains("No locations found"));
            }
            _ => panic!("Expected GeocodingError, got {:?}", err),
        }
    }
}

#[tokio::test]
async fn test_weather_api_failure() {
    let mut mock_client = MockClient::new();

    // Mock successful geocoding
    mock_client.mock_response("geocoding-api.open-meteo.com", json!({
        "results": [{
            "latitude": 40.7128,
            "longitude": -74.0060,
            "timezone": "America/New_York"
        }]
    }));

    // For API failure, we don't mock the weather API response, so it will fail
    let http_client = HttpClient::Mock(mock_client);

    let request = aws_lambda_mcp::models::WeatherRequest {
        location: "New York".to_string(),
    };

    let result = get_weather_with_client(&http_client, request).await;
    println!("Result is err: {}", result.is_err());
    if result.is_ok() {
        println!("Unexpected success: {:?}", result.unwrap());
        panic!("Expected error but got success");
    }
    let err = result.unwrap_err();
    println!("Error type: {:?}", err);
    match err {
        aws_lambda_mcp::models::AppError::WeatherApiError(msg) => {
            assert!(msg.contains("No mock response configured"));
        }
        _ => panic!("Expected WeatherApiError, got: {:?}", err),
    }
}

#[tokio::test]
async fn test_weather_malformed_geocoding_response() {
    let mut mock_client = MockClient::new();

    // Mock malformed geocoding response (missing required fields)
    mock_client.mock_response("geocoding-api.open-meteo.com", json!({"results": [{"invalid_field": "value"}] }));

    let http_client = HttpClient::Mock(mock_client);

    let request = aws_lambda_mcp::models::WeatherRequest {
        location: "New York".to_string(),
    };

    let result = get_weather_with_client(&http_client, request).await;
    assert!(result.is_err(), "Should fail with malformed geocoding response");

    if let Err(err) = result {
        match err {
            aws_lambda_mcp::models::AppError::GeocodingError(msg) => {
                assert!(msg.contains("Failed to extract latitude"));
            }
            _ => panic!("Expected GeocodingError, got {:?}", err),
        }
    }
}

#[tokio::test]
async fn test_weather_malformed_weather_response() {
    let mut mock_client = MockClient::new();

    // Mock successful geocoding
    mock_client.mock_response("geocoding-api.open-meteo.com", json!({
        "results": [{
            "latitude": 40.7128,
            "longitude": -74.0060,
            "timezone": "America/New_York"
        }]
    }));

    // Mock malformed weather response
    mock_client.mock_response("api.open-meteo.com", json!({"invalid": "response"}));

    let http_client = HttpClient::Mock(mock_client);

    let request = aws_lambda_mcp::models::WeatherRequest {
        location: "New York".to_string(),
    };

    let result = get_weather_with_client(&http_client, request).await;
    assert!(result.is_err(), "Should fail with malformed weather response");

    if let Err(err) = result {
        match err {
            aws_lambda_mcp::models::AppError::WeatherApiError(msg) => {
                assert!(msg.contains("Failed to parse weather forecast response"));
            }
            _ => panic!("Expected WeatherApiError, got {:?}", err),
        }
    } else {
        panic!("Expected error but got success");
    }
}

#[tokio::test]
async fn test_route_tool_malformed_json() {
    // Test with completely malformed JSON
    let malformed_payload = json!({"invalid": "json", "missing": "method"});

    let result = route_tool("get_weather", malformed_payload).await;
    assert!(result.is_err(), "Should fail with malformed JSON");

    if let Err(err) = result {
        assert_eq!(err.error_type, "InvalidInput");
    }
}

#[tokio::test]
async fn test_route_tool_empty_payload() {
    // Test with empty payload
    let empty_payload = json!({});

    let result = route_tool("get_weather", empty_payload).await;
    assert!(result.is_err(), "Should fail with empty payload");

    if let Err(err) = result {
        assert_eq!(err.error_type, "InvalidInput");
    }
}

#[tokio::test]
async fn test_route_tool_null_arguments() {
    // Test with null arguments
    let null_payload = json!({
        "method": "tools/call",
        "params": {
            "arguments": null
        }
    });

    let result = route_tool("get_weather", null_payload).await;
    assert!(result.is_err(), "Should fail with null arguments");

    if let Err(err) = result {
        assert_eq!(err.error_type, "InvalidInput");
    }
}

#[test]
fn test_schema_generation() {
    // Run the schema generation binary
    let output = Command::new("cargo")
        .args(["run", "--bin", "generate-schema", "--features", "schema-gen"])
        .output()
        .expect("Failed to run schema generation");

    assert!(output.status.success(), "Schema generation should succeed");

    // Check that tool_schema.json was created
    assert!(fs::metadata("tool_schema.json").is_ok(), "tool_schema.json should be created");

    // Read and validate the schema content
    let schema_content = fs::read_to_string("tool_schema.json")
        .expect("Failed to read tool_schema.json");

    let schema: serde_json::Value = serde_json::from_str(&schema_content)
        .expect("Schema should be valid JSON");

    // Validate schema structure
    let tools = schema.as_array()
        .expect("Schema should be an array of tools");

    assert!(!tools.is_empty(), "Should have at least one tool");

    // Check that both expected tools are present
    let tool_names: Vec<&str> = tools.iter()
        .filter_map(|tool| tool.get("name").and_then(|n| n.as_str()))
        .collect();

    assert!(tool_names.contains(&"get_weather"), "Should contain get_weather tool");
    assert!(tool_names.contains(&"get_personalized_greeting"), "Should contain get_personalized_greeting tool");

    // Validate weather tool schema
    let weather_tool = tools.iter()
        .find(|tool| tool.get("name").and_then(|n| n.as_str()) == Some("get_weather"))
        .expect("Should find weather tool");

    assert!(weather_tool.get("inputSchema").is_some(), "Weather tool should have input schema");
    assert!(weather_tool.get("outputSchema").is_some(), "Weather tool should have output schema");

    // Validate greeting tool schema
    let greeting_tool = tools.iter()
        .find(|tool| tool.get("name").and_then(|n| n.as_str()) == Some("get_personalized_greeting"))
        .expect("Should find greeting tool");

    assert!(greeting_tool.get("inputSchema").is_some(), "Greeting tool should have input schema");
    assert!(greeting_tool.get("outputSchema").is_some(), "Greeting tool should have output schema");

    // Clean up
    fs::remove_file("tool_schema.json").expect("Failed to clean up test file");
}

#[test]
fn test_schema_validation_weather_request() {
    // Test that WeatherRequest can be serialized/deserialized according to schema
    let request = aws_lambda_mcp::models::WeatherRequest {
        location: "Test City".to_string(),
    };

    let json_value = serde_json::to_value(&request)
        .expect("WeatherRequest should serialize to JSON");

    // Validate required fields are present
    assert!(json_value.get("location").is_some(), "Should have location field");

    // Test deserialization
    let deserialized: aws_lambda_mcp::models::WeatherRequest = serde_json::from_value(json_value)
        .expect("Should deserialize back to WeatherRequest");

    assert_eq!(deserialized.location, "Test City");
}

#[test]
fn test_schema_validation_greeting_request() {
    // Test that PersonalizedGreetingRequest can be serialized/deserialized
    let request = aws_lambda_mcp::models::PersonalizedGreetingRequest {
        user_id: "test@example.com".to_string(),
        user_name: "Test User".to_string(),
    };

    let json_value = serde_json::to_value(&request)
        .expect("PersonalizedGreetingRequest should serialize to JSON");

    // Validate fields are present (note: serde uses snake_case by default)
    assert!(json_value.get("user_id").is_some(), "Should have user_id field");
    assert!(json_value.get("user_name").is_some(), "Should have user_name field");

    // Test deserialization
    let deserialized: aws_lambda_mcp::models::PersonalizedGreetingRequest = serde_json::from_value(json_value)
        .expect("Should deserialize back to PersonalizedGreetingRequest");

    assert_eq!(deserialized.user_id, "test@example.com");
    assert_eq!(deserialized.user_name, "Test User");
}

/// Helper function to assert successful greeting response
fn assert_successful_greeting(
    result: Result<serde_json::Value, lambda_runtime::Diagnostic>,
    expected_name: &str,
) {
    assert!(result.is_ok(), "Expected successful greeting");

    if let Ok(response) = result {
        let greeting = response.get("greeting").and_then(|g| g.as_str());
        assert!(greeting.is_some(), "Response should contain greeting field");

        if let Some(greeting_text) = greeting {
            assert!(
                greeting_text.contains(expected_name),
                "Greeting should contain the expected name '{expected_name}', but was '{greeting_text}'"
            );
        }
    }
}
