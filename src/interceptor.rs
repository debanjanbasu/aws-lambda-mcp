use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simplified interceptor event structure - only handles REQUEST interception
/// This represents the event sent to the interceptor Lambda from API Gateway
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InterceptorEvent {
    pub gateway_request: GatewayRequest,
}

/// Gateway request structure containing headers and body
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GatewayRequest {
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

/// Simplified interceptor response - only transforms requests
/// This is the response sent back to API Gateway to modify the request
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InterceptorResponse {
    pub version: String,
    pub transformed_gateway_request: GatewayRequest,
}

/// Header key constants for maintainability
const AUTH_HEADER: &str = "authorization";
const CUSTOM_HEADER: &str = "customHeaderKey";

/// Extract authorization token from headers
///
/// # Arguments
/// * `headers` - The request headers
///
/// # Returns
/// The authorization header value if present
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn extract_auth_header(headers: &HashMap<String, String>) -> Option<String> {
    headers.get(AUTH_HEADER).cloned()
}

/// Extract custom header for propagation
///
/// # Arguments
/// * `headers` - The request headers
///
/// # Returns
/// The custom header value if present
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn extract_custom_header(headers: &HashMap<String, String>) -> Option<String> {
    headers.get(CUSTOM_HEADER).cloned()
}

/// Perform token exchange (placeholder implementation)
///
/// This is a placeholder that should be replaced with actual token exchange logic
/// such as calling an OAuth endpoint or validating the token.
///
/// # Arguments
/// * `_auth_token` - The original authorization token
///
/// # Returns
/// A placeholder exchanged token string
#[must_use]
pub fn exchange_token(_auth_token: &str) -> String {
    "exchanged_token_placeholder".to_string()
}

/// Process interceptor event to transform the gateway request
///
/// This function extracts authorization and custom headers, performs token exchange,
/// and modifies the request body to include the exchanged credentials and custom headers.
///
/// # Arguments
/// * `event` - The interceptor event containing the gateway request
///
/// # Returns
/// The interceptor response with the transformed request
pub fn process_interceptor_event(mut event: InterceptorEvent) -> InterceptorResponse {
    // Process headers if they exist
    if let Some(ref headers) = event.gateway_request.headers {
        // Extract tokens and headers
        let auth_token = extract_auth_header(headers);
        let custom_header = extract_custom_header(headers);
        let exchanged_credentials = auth_token.as_ref().map(|token| exchange_token(token));

        // Modify request body to include extracted data
        if let Some(ref mut body_str) = event.gateway_request.body {
            match serde_json::from_str::<serde_json::Value>(body_str) {
                Ok(mut body) => {
                    if let Some(params) = body.get_mut("params")
                        && let Some(args) = params.get_mut("arguments")
                        && let Some(args_obj) = args.as_object_mut() {
                        // Add exchanged credentials if we have an auth token
                        if let Some(creds) = exchanged_credentials {
                            args_obj.insert("exchanged_credentials".to_string(), serde_json::json!(creds));
                        }
                        // Add custom header if present
                        if let Some(custom) = custom_header {
                            args_obj.insert(CUSTOM_HEADER.to_string(), serde_json::json!(custom));
                        }
                        // Update the body
                        *body_str = body.to_string();
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to parse request body as JSON. Body will be passed through unmodified.");
                }
            }
        }
    }

    InterceptorResponse {
        version: "1.0".to_string(),
        transformed_gateway_request: event.gateway_request,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_auth_header() {
        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "Bearer token123".to_string());
        headers.insert("content-type".to_string(), "application/json".to_string());

        assert_eq!(extract_auth_header(&headers), Some("Bearer token123".to_string()));
    }

    #[test]
    fn test_extract_auth_header_missing() {
        let headers = HashMap::new();
        assert_eq!(extract_auth_header(&headers), None);
    }

    #[test]
    fn test_extract_custom_header() {
        let mut headers = HashMap::new();
        headers.insert("customHeaderKey".to_string(), "custom_value".to_string());

        assert_eq!(extract_custom_header(&headers), Some("custom_value".to_string()));
    }

    #[test]
    fn test_extract_custom_header_missing() {
        let headers = HashMap::new();
        assert_eq!(extract_custom_header(&headers), None);
    }

    #[test]
    fn test_exchange_token() {
        // Test the placeholder implementation
        assert_eq!(exchange_token("some_token"), "exchanged_token_placeholder".to_string());
    }

    #[test]
    fn test_process_interceptor_event_with_auth_and_custom() {
        let event = InterceptorEvent {
            gateway_request: GatewayRequest {
                headers: Some({
                    let mut h = HashMap::new();
                    h.insert("authorization".to_string(), "Bearer test_token".to_string());
                    h.insert("customHeaderKey".to_string(), "custom_val".to_string());
                    h
                }),
                body: Some(json!({
                    "jsonrpc": "2.0",
                    "method": "tools/call",
                    "params": {
                        "name": "test_tool",
                        "arguments": {}
                    }
                }).to_string()),
            },
        };

        let response = process_interceptor_event(event);
        assert_eq!(response.version, "1.0");

        // Check that body was modified
        let body = response.transformed_gateway_request.body.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        let args = &parsed["params"]["arguments"];

        // Should have added exchanged_credentials and customHeaderKey
        assert_eq!(args["exchanged_credentials"], "exchanged_token_placeholder");
        assert_eq!(args["customHeaderKey"], "custom_val");
    }

    #[test]
    fn test_process_interceptor_event_no_headers() {
        let event = InterceptorEvent {
            gateway_request: GatewayRequest {
                headers: None,
                body: Some(json!({"test": "data"}).to_string()),
            },
        };

        let response = process_interceptor_event(event);
        // Body should be unchanged
        assert_eq!(response.transformed_gateway_request.body, Some(json!({"test": "data"}).to_string()));
    }

    #[test]
    fn test_process_interceptor_event_invalid_json_body() {
        let event = InterceptorEvent {
            gateway_request: GatewayRequest {
                headers: Some({
                    let mut h = HashMap::new();
                    h.insert("authorization".to_string(), "Bearer token".to_string());
                    h
                }),
                body: Some("invalid json".to_string()),
            },
        };

        let response = process_interceptor_event(event);
        // Body should be passed through unchanged
        assert_eq!(response.transformed_gateway_request.body, Some("invalid json".to_string()));
    }
}