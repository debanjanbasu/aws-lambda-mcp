// Handler tests
#![allow(clippy::unwrap_used)]

use aws_lambda_mcp::handler::route_tool;
use serde_json::json;

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

    // This test verifies that arguments are correctly parsed.
    // It may succeed (if network is available) or fail with a ToolError (if network is blocked).
    let result = route_tool("get_weather", mcp_payload).await;

    match result {
        Ok(_) => {
            // Success is fine, it means arguments were parsed and the API call worked.
        }
        Err(err) => {
            // If it fails, it should be a ToolError (parsing succeeded, API call failed),
            // not an InvalidInput error (parsing failed).
            assert_eq!(err.error_type, "ToolError");
        }
    }
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
