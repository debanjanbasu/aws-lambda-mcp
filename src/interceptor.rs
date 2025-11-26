use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, info};

// Header key constants for maintainability
pub const AUTH_HEADER: &str = "authorization";
pub const CUSTOM_HEADER: &str = "customHeaderKey";

/// Simplified interceptor event structure - only handles REQUEST interception
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InterceptorEvent {
    pub gateway_request: GatewayRequest,
}

/// Gateway request structure
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GatewayRequest {
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

/// Simplified interceptor response - only transforms requests
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InterceptorResponse {
    pub version: String,
    pub transformed_gateway_request: GatewayRequest,
}

/// Extract authorization token from headers
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn extract_auth_header(headers: &HashMap<String, String>) -> Option<String> {
    headers.get(AUTH_HEADER).cloned()
}

/// Extract custom header for propagation
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn extract_custom_header(headers: &HashMap<String, String>) -> Option<String> {
    headers.get(CUSTOM_HEADER).cloned()
}

/// Perform token exchange (placeholder)
#[must_use]
pub fn exchange_token(_auth_token: &str) -> String {
    "exchanged_token_placeholder".to_string()
}

/// Process interceptor request and return transformed response
pub fn process_interceptor_request(event: InterceptorEvent) -> InterceptorResponse {
    let mut gateway_request = event.gateway_request;

    info!("Processing gateway request interceptor");

    // Process headers if they exist
    if let Some(ref headers) = gateway_request.headers {
        debug!("Processing {} headers", headers.len());

        // Extract tokens and headers
        let auth_token = extract_auth_header(headers);
        let custom_header = extract_custom_header(headers);
        let exchanged_credentials = auth_token.as_ref().map(|token| exchange_token(token));

        // Modify request body to include extracted data
        if let Some(ref mut body_str) = gateway_request.body {
            match serde_json::from_str::<Value>(body_str) {
                Ok(mut body) => {
                    if let Some(params) = body.get_mut("params")
                        && let Some(args) = params.get_mut("arguments")
                        && let Some(args_obj) = args.as_object_mut() {
                        // Add exchanged credentials if we have an auth token
                        if let Some(creds) = exchanged_credentials {
                            args_obj.insert("exchanged_credentials".to_string(), json!(creds));
                        }
                        // Add custom header if present
                        if let Some(custom) = custom_header {
                            args_obj.insert(CUSTOM_HEADER.to_string(), json!(custom));
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

    info!("Request processing complete");
    InterceptorResponse {
        version: "1.0".to_string(),
        transformed_gateway_request: gateway_request,
    }
}