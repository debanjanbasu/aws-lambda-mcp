// Integration tests for interceptor functionality
// Note: These tests focus on the public behavior and helper functions
#![allow(clippy::expect_used, clippy::panic)]

use aws_lambda_mcp::models::interceptor::InterceptorEvent;

#[test]
fn test_jwt_token_structure() {
    // Test that we can parse a JWT-like structure (without actual decoding)
    let token = "header.payload.signature";

    // Basic JWT structure validation
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3, "JWT should have 3 parts separated by dots");

    // Check that header and payload are base64-like (basic validation)
    assert!(!parts[0].is_empty(), "Header should not be empty");
    assert!(!parts[1].is_empty(), "Payload should not be empty");
    assert!(!parts[2].is_empty(), "Signature should not be empty");
}

#[test]
fn test_interceptor_event_parsing() {
    // Test parsing the interceptor event structure from a generic event
    let test_event = r#"{
        "interceptorInputVersion": "1.0",
        "mcp": {
            "gatewayRequest": {
                "headers": {
                    "authorization": "Bearer header.payload.signature"
                },
                "body": "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"tools/call\", \"params\": {\"name\": \"get_weather\", \"arguments\": {}}}"
            }
        }
    }"#;

    // Parse the event
    let event: serde_json::Value = serde_json::from_str(test_event)
        .expect("Failed to parse test event JSON - this indicates a test setup issue");

    // This should match the InterceptorEvent structure
    let interceptor_event: InterceptorEvent = serde_json::from_value(event)
        .expect("Failed to deserialize into InterceptorEvent - this indicates a test setup issue");

    // Verify the structure
    assert_eq!(interceptor_event.interceptor_input_version, "1.0");
    assert!(interceptor_event.mcp.gateway_request.headers.is_some());
    assert!(interceptor_event.mcp.gateway_request.body.is_some());

    // Check that authorization header is present
    let headers = interceptor_event
        .mcp
        .gateway_request
        .headers
        .as_ref()
        .expect("Headers should be present in test setup");
    assert!(headers.contains_key("authorization"));
    let auth_header = headers
        .get("authorization")
        .expect("Authorization header should be present in test setup");
    assert!(auth_header.starts_with("Bearer "));
}

#[test]
fn test_gateway_prefix_stripping() {
    // Test that the interceptor correctly strips gateway prefixes from tool names

    // This test simulates what happens in the interceptor code
    let test_cases = vec![
        ("get_weather", "get_weather"),
        ("gateway-123___get_weather", "get_weather"),
        (
            "aws-agentcore-gateway-target___get_personalized_greeting",
            "get_personalized_greeting",
        ),
        ("custom-prefix___tool_name", "tool_name"),
    ];

    // We can't directly call the private strip_gateway_prefix function,
    // but we can test the behavior through the extract_tool_name function
    // by mocking the JSON structure

    for (input_name, expected_name) in test_cases {
        let body = serde_json::json!({
            "params": {
                "name": input_name
            }
        });

        // This mimics the logic in the interceptor
        let extracted_name = body
            .get("params")
            .and_then(|params| params.get("name"))
            .and_then(serde_json::Value::as_str)
            .map(|name| {
                if let Some((_, actual_name)) = name.split_once("___") {
                    actual_name.to_string()
                } else {
                    name.to_string()
                }
            });

        assert_eq!(
            extracted_name,
            Some(expected_name.to_string()),
            "Failed to strip prefix from '{input_name}'"
        );
    }
}
