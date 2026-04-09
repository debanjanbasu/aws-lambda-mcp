// Integration tests for the full request flow
// These tests verify the complete end-to-end functionality
#![allow(clippy::unwrap_used)]

use aws_lambda_mcp::handler::{function_handler, extract_tool_name};
use aws_lambda_mcp::models::interceptor::{InterceptorEvent, GatewayRequest};
use lambda_runtime::{Context, LambdaEvent};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_full_weather_request_flow() {
    // Create a complete Lambda event simulating a weather request
    let event_payload = json!({
        "method": "tools/call",
        "params": {
            "name": "get_weather",
            "arguments": {
                "location": "London"
            }
        }
    });

    let lambda_event = create_test_lambda_event(event_payload);

    // Note: This test will make real HTTP calls, so it may fail if network is unavailable
    // In a real CI environment, this would be mocked or skipped
    let result = function_handler(lambda_event).await;

    // The result should either succeed (if network works) or fail with ToolError (if network fails)
    match result {
        Ok(_) => {
            // Success means the full flow worked
            println!("Full weather request flow succeeded");
        }
        Err(diagnostic) => {
            // Should be a ToolError, not an InvalidInput or other error
            assert_eq!(diagnostic.error_type, "ToolError",
                "Expected ToolError for network/API issues, got: {}", diagnostic.error_type);
            println!("Weather request failed as expected due to network/API issues: {}", diagnostic.error_message);
        }
    }
}

#[tokio::test]
async fn test_full_greeting_request_flow() {
    // Create a complete Lambda event simulating a greeting request
    let event_payload = json!({
        "method": "tools/call",
        "params": {
            "name": "get_personalized_greeting",
            "arguments": {
                "user_name": "Alice",
                "user_id": "alice@example.com"
            }
        }
    });

    let lambda_event = create_test_lambda_event(event_payload);

    let result = function_handler(lambda_event).await;

    // Greeting should always succeed (no external dependencies)
    assert!(result.is_ok(), "Greeting request should always succeed");

    let response = result.unwrap();
    let greeting = response.get("greeting").and_then(|g| g.as_str());
    assert!(greeting.is_some(), "Response should contain greeting field");
    assert!(greeting.unwrap().contains("Alice"), "Greeting should contain user name");
}

#[tokio::test]
async fn test_tool_name_extraction_from_context() {
    // Test extraction from Lambda context (Bedrock AgentCore style)
    // Note: Context creation is complex, so we'll test the payload extraction instead
    let event_payload = json!({
        "method": "tools/call",
        "params": {
            "name": "get_weather"
        }
    });

    // Create a minimal context for testing
    let context = lambda_runtime::Context::default();

    let tool_name = extract_tool_name(&event_payload, &context);
    assert_eq!(tool_name, "get_weather");
}

#[tokio::test]
async fn test_tool_name_extraction_from_mcp_payload() {
    // Test extraction from MCP payload
    let event_payload = json!({
        "method": "tools/call",
        "params": {
            "name": "get_personalized_greeting"
        }
    });

    let context = Context::default(); // No custom context

    let tool_name = extract_tool_name(&event_payload, &context);
    assert_eq!(tool_name, "get_personalized_greeting");
}

#[tokio::test]
async fn test_tool_name_extraction_with_gateway_prefix() {
    // Test extraction with gateway prefix stripping
    let event_payload = json!({
        "method": "tools/call",
        "params": {
            "name": "aws-agentcore-gateway-target___get_weather"
        }
    });

    let context = Context::default();

    let tool_name = extract_tool_name(&event_payload, &context);
    assert_eq!(tool_name, "get_weather");
}

#[tokio::test]
async fn test_unknown_tool_flow() {
    // Test the full flow with an unknown tool
    let event_payload = json!({
        "method": "tools/call",
        "params": {
            "name": "unknown_tool",
            "arguments": {}
        }
    });

    let lambda_event = create_test_lambda_event(event_payload);

    let result = function_handler(lambda_event).await;
    assert!(result.is_err(), "Unknown tool should result in error");

    if let Err(diagnostic) = result {
        assert_eq!(diagnostic.error_type, "UnknownTool");
        assert!(diagnostic.error_message.contains("unknown_tool"));
    }
}

#[tokio::test]
async fn test_api_gateway_event_format() {
    // Test handling of API Gateway event format (body field)
    let api_gateway_payload = json!({
        "body": "{\"method\": \"tools/call\", \"params\": {\"name\": \"get_personalized_greeting\", \"arguments\": {\"user_name\": \"Bob\"}}}"
    });

    let lambda_event = create_test_lambda_event(api_gateway_payload);

    let result = function_handler(lambda_event).await;
    assert!(result.is_ok(), "API Gateway format should be handled correctly");

    let response = result.unwrap();
    let greeting = response.get("greeting").and_then(|g| g.as_str());
    assert!(greeting.is_some(), "Response should contain greeting field");
    assert!(greeting.unwrap().contains("Bob"), "Greeting should contain user name");
}

#[tokio::test]
async fn test_malformed_api_gateway_body() {
    // Test handling of malformed API Gateway body
    let api_gateway_payload = json!({
        "body": "invalid json"
    });

    let lambda_event = create_test_lambda_event(api_gateway_payload);

    let result = function_handler(lambda_event).await;
    assert!(result.is_err(), "Malformed API Gateway body should result in error");
}

#[tokio::test]
async fn test_concurrent_greeting_requests() {
    // Test concurrent execution of greeting requests
    let mut handles = vec![];

    for i in 0..10 {
        let user_name = format!("User{}", i);
        let handle = tokio::spawn(async move {
            let event_payload = json!({
                "method": "tools/call",
                "params": {
                    "name": "get_personalized_greeting",
                    "arguments": {
                        "user_name": user_name
                    }
                }
            });

            let lambda_event = create_test_lambda_event(event_payload);
            function_handler(lambda_event).await
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let results: Vec<Result<Result<serde_json::Value, lambda_runtime::Diagnostic>, tokio::task::JoinError>> = futures::future::join_all(handles).await;

    // All should succeed
    for result in results {
        let response: Result<serde_json::Value, lambda_runtime::Diagnostic> = result.expect("Task should not panic");
        assert!(response.is_ok(), "Concurrent greeting request should succeed");

        let greeting_response = response.unwrap();
        let greeting = greeting_response.get("greeting").and_then(|g: &serde_json::Value| g.as_str());
        assert!(greeting.is_some(), "Response should contain greeting field");
    }
}

#[tokio::test]
async fn test_request_response_performance() {
    // Test that requests complete within reasonable time bounds
    use std::time::Instant;

    let event_payload = json!({
        "method": "tools/call",
        "params": {
            "name": "get_personalized_greeting",
            "arguments": {
                "user_name": "PerformanceTest"
            }
        }
    });

    let lambda_event = create_test_lambda_event(event_payload);

    let start = Instant::now();
    let result = function_handler(lambda_event).await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Request should succeed");
    assert!(duration.as_millis() < 100, "Request should complete within 100ms, took {}ms", duration.as_millis());
}

/// Helper function to create a test Lambda event
fn create_test_lambda_event(payload: serde_json::Value) -> LambdaEvent<serde_json::Value> {
    LambdaEvent {
        payload,
        context: Context::default(),
    }
}