// Handler tests
#![allow(clippy::unwrap_used)]

use async_trait::async_trait;
use aws_lambda_mcp::handler::route_tool;
use aws_lambda_mcp::http::HttpClient;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;

/// Mock HTTP client for testing
struct MockHttpClient {
    responses: Mutex<HashMap<String, serde_json::Value>>,
}

impl MockHttpClient {
    fn new() -> Self {
        let mut responses = HashMap::new();

        // Mock geocoding response for "New York"
        responses.insert(
            "https://geocoding-api.open-meteo.com/v1/search?name=New%20York&count=1&language=en&format=json".to_string(),
            json!({
                "results": [{
                    "latitude": 40.7128,
                    "longitude": -74.0060,
                    "timezone": "America/New_York"
                }]
            })
        );

        // Mock weather response
        responses.insert(
            "https://api.open-meteo.com/v1/forecast?latitude=40.7128&longitude=-74.0060&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=America/New_York".to_string(),
            json!({
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
                    "weather_code": [0],
                    "temperature_2m_max": [5.0],
                    "temperature_2m_min": [-2.0]
                }
            })
        );

        Self {
            responses: Mutex::new(responses),
        }
    }
}

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn get(&self, _url: &str) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        // For simplicity, we'll just return an error since we don't need this for weather tests
        Err("Mock get not implemented".into())
    }

    async fn get_json_value(&self, url: &str) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let responses = self.responses.lock().unwrap();

        // Check for exact match first
        if let Some(response) = responses.get(url) {
            return Ok(response.clone());
        }

        // For geocoding, check if URL contains the key parts
        if url.contains("geocoding-api.open-meteo.com") && url.contains("name=New%20York") {
            if let Some(response) = responses.get(&"https://geocoding-api.open-meteo.com/v1/search?name=New%20York&count=1&language=en&format=json".to_string()) {
                return Ok(response.clone());
            }
        }

        // For weather, check if URL contains the key parts
        if url.contains("api.open-meteo.com") && url.contains("latitude=40.7128") && url.contains("longitude=-74.006") {
            if let Some(response) = responses.get(&"https://api.open-meteo.com/v1/forecast?latitude=40.7128&longitude=-74.0060&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=America/New_York".to_string()) {
                return Ok(response.clone());
            }
        }

        Err(format!("No mock response for URL: {}", url).into())
    }
}

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
async fn test_weather_argument_extraction() {
    // Simulate MCP request structure with arguments for get_weather
    let mcp_payload = json!({
        "method": "tools/call",
        "params": {
            "arguments": {
                "location": "New York"
            }
        }
    });

    // Use mock client for testing
    let mock_client = MockHttpClient::new();
    let result = aws_lambda_mcp::handler::route_tool_with_client("get_weather", mcp_payload, &mock_client).await;

    // With mock client, this should succeed
    assert!(result.is_ok(), "Weather request should succeed with mock client");
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

/// Helper function to assert successful greeting response
fn assert_successful_greeting(result: Result<serde_json::Value, lambda_runtime::Diagnostic>, expected_name: &str) {
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
