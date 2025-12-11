// Handler tests
#![allow(clippy::unwrap_used)]

use aws_lambda_mcp::handler::route_tool;
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

#[tokio::test]
async fn test_tool_schema_generation() {
    // Run the generate-schema binary
    let output = Command::new("cargo")
        .args(&["run", "--bin", "generate-schema", "--features", "schema-gen"])
        .output()
        .expect("Failed to run generate-schema");

    assert!(output.status.success(), "generate-schema failed: {:?}", output);

    // Check that tool_schema.json was created
    assert!(fs::metadata("tool_schema.json").is_ok(), "tool_schema.json was not created");

    // Read and parse the schema
    let content = fs::read_to_string("tool_schema.json").expect("Failed to read tool_schema.json");
    let schema: serde_json::Value = serde_json::from_str(&content).expect("tool_schema.json is not valid JSON");

    // Check that it's an array with 2 tools
    assert!(schema.is_array(), "Schema should be an array");
    let tools = schema.as_array().unwrap();
    assert_eq!(tools.len(), 2, "Should have 2 tools");

    // Check the tool names
    let tool_names: Vec<&str> = tools.iter()
        .map(|t| t.get("name").and_then(|n| n.as_str()).unwrap())
        .collect();
    assert!(tool_names.contains(&"get_weather"), "Should contain get_weather tool");
    assert!(tool_names.contains(&"get_personalized_greeting"), "Should contain get_personalized_greeting tool");

    // Check that each tool has the required fields
    for tool in tools {
        assert!(tool.get("name").is_some(), "Tool should have name");
        assert!(tool.get("description").is_some(), "Tool should have description");
        assert!(tool.get("inputSchema").is_some(), "Tool should have inputSchema");
        assert!(tool.get("outputSchema").is_some(), "Tool should have outputSchema");
    }
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
