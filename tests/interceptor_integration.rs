use aws_lambda_mcp::interceptor::{process_interceptor_request, GatewayRequest, InterceptorEvent};
use serde_json::{json, Value};
use std::collections::HashMap;

#[test]
fn test_interceptor_with_auth_header_and_custom_header() {
    let event = InterceptorEvent {
        gateway_request: GatewayRequest {
            headers: Some(HashMap::from([
                ("authorization".to_string(), "Bearer token123".to_string()),
                ("customHeaderKey".to_string(), "custom-value".to_string()),
                ("content-type".to_string(), "application/json".to_string()),
            ])),
            body: Some(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "get_weather",
                    "arguments": {
                        "location": "Sydney"
                    }
                }
            }).to_string()),
        },
    };

    let response = process_interceptor_request(event);
    assert_eq!(response.version, "1.0");

    // Check that the body was modified to include exchanged credentials and custom header
    let body: Value = serde_json::from_str(&response.transformed_gateway_request.body.unwrap()).unwrap();
    let params = body.get("params").unwrap();
    let args = params.get("arguments").unwrap();
    let args_obj = args.as_object().unwrap();

    // Should have exchanged credentials
    assert!(args_obj.contains_key("exchanged_credentials"));
    assert_eq!(args_obj.get("exchanged_credentials").unwrap(), "exchanged_token_placeholder");

    // Should have custom header
    assert!(args_obj.contains_key("customHeaderKey"));
    assert_eq!(args_obj.get("customHeaderKey").unwrap(), "custom-value");
}

#[test]
fn test_interceptor_with_only_auth_header() {
    let event = InterceptorEvent {
        gateway_request: GatewayRequest {
            headers: Some(HashMap::from([
                ("authorization".to_string(), "Bearer token456".to_string()),
            ])),
            body: Some(json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "get_weather",
                    "arguments": {
                        "location": "London"
                    }
                }
            }).to_string()),
        },
    };

    let response = process_interceptor_request(event);

    let body: Value = serde_json::from_str(&response.transformed_gateway_request.body.unwrap()).unwrap();
    let params = body.get("params").unwrap();
    let args = params.get("arguments").unwrap();
    let args_obj = args.as_object().unwrap();

    // Should have exchanged credentials
    assert!(args_obj.contains_key("exchanged_credentials"));
    assert_eq!(args_obj.get("exchanged_credentials").unwrap(), "exchanged_token_placeholder");

    // Should not have custom header since it wasn't provided
    assert!(!args_obj.contains_key("customHeaderKey"));
}

#[test]
fn test_interceptor_with_no_headers() {
    let event = InterceptorEvent {
        gateway_request: GatewayRequest {
            headers: None,
            body: Some(json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "get_weather",
                    "arguments": {
                        "location": "Tokyo"
                    }
                }
            }).to_string()),
        },
    };

    let response = process_interceptor_request(event);

    let body: Value = serde_json::from_str(&response.transformed_gateway_request.body.unwrap()).unwrap();
    let params = body.get("params").unwrap();
    let args = params.get("arguments").unwrap();
    let args_obj = args.as_object().unwrap();

    // Should not have exchanged credentials or custom header
    assert!(!args_obj.contains_key("exchanged_credentials"));
    assert!(!args_obj.contains_key("customHeaderKey"));
}

#[test]
fn test_interceptor_with_malformed_json_body() {
    let event = InterceptorEvent {
        gateway_request: GatewayRequest {
            headers: Some(HashMap::from([
                ("authorization".to_string(), "Bearer token789".to_string()),
            ])),
            body: Some("invalid json".to_string()),
        },
    };

    let response = process_interceptor_request(event);

    // Body should be passed through unchanged
    assert_eq!(response.transformed_gateway_request.body.unwrap(), "invalid json");
}

#[test]
fn test_interceptor_with_no_body() {
    let event = InterceptorEvent {
        gateway_request: GatewayRequest {
            headers: Some(HashMap::from([
                ("authorization".to_string(), "Bearer token999".to_string()),
            ])),
            body: None,
        },
    };

    let response = process_interceptor_request(event);

    // Body should remain None
    assert!(response.transformed_gateway_request.body.is_none());
}