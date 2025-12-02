// Integration tests for interceptor functionality
// Note: These tests focus on the public behavior and helper functions

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

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
        .expect("Failed to parse test event JSON");

    // This should match the InterceptorEvent structure
    let interceptor_event: InterceptorEvent = serde_json::from_value(event.clone())
        .expect("Failed to deserialize into InterceptorEvent");

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
        .expect("Headers should be present");
    assert!(headers.contains_key("authorization"));
    let auth_header = headers
        .get("authorization")
        .expect("Authorization header should be present");
    assert!(auth_header.starts_with("Bearer "));
}
