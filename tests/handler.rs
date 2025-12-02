use aws_lambda_mcp::handler::{route_tool, strip_gateway_prefix};
use serde_json::json;

#[test]
fn test_strip_gateway_prefix() {
    assert_eq!(
        strip_gateway_prefix("gateway-123___get_weather"),
        "get_weather"
    );
    assert_eq!(strip_gateway_prefix("get_weather"), "get_weather");
    assert_eq!(strip_gateway_prefix(""), "");
    assert_eq!(strip_gateway_prefix("no_prefix"), "no_prefix");
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
                "latitude": 48.8566,
                "longitude": 2.3522,
                "daily": ["weather_code", "temperature_2m_max", "temperature_2m_min"],
                "timezone": "Europe/Berlin"
            }
        }
    });

    // This test verifies that arguments are correctly parsed.
    // It may succeed (if network is available) or fail with a ToolError (if network is blocked).
    let result = route_tool("get_weather", mcp_payload).await;

    match result {
        Ok(_) => {
            // Success is fine, it means arguments were parsed and the API call worked.
            assert!(true);
        }
        Err(err) => {
            // If it fails, it should be a ToolError (parsing succeeded, API call failed),
            // not an InvalidInput error (parsing failed).
            assert_eq!(err.error_type, "ToolError");

            // Optionally, we can check for network-related errors, but success is also OK.
            // The main point is that argument parsing worked.
            // If we get here, the test has effectively passed its primary goal.
        }
    }
}
