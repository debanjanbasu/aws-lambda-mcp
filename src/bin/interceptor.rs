use lambda_runtime::{Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use lambda_runtime::tracing::{debug, info};
use serde::Deserialize as SerdeDeserialize;
use base64::{engine::general_purpose, Engine as _};

// Header key constants for maintainability
const AUTH_HEADER: &str = "authorization";
const CUSTOM_HEADER: &str = "customHeaderKey";

/// Minimal JWT claims for extracting user information
#[derive(SerdeDeserialize, Debug)]
struct JwtClaims {
    #[serde(default)]
    sub: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    preferred_username: String,
}

/// Interceptor event structure matching AWS Bedrock `AgentCore` Gateway specification
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InterceptorEvent {
    #[allow(dead_code)]
    interceptor_input_version: String,
    mcp: McpData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct McpData {
    gateway_request: GatewayRequest,
}

/// Gateway request structure for interceptor response
/// Only includes headers and body as per AWS spec
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct GatewayRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<Value>,
}

/// Interceptor response matching AWS Bedrock `AgentCore` Gateway specification
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InterceptorResponse {
    interceptor_output_version: String,
    mcp: McpResponse,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct McpResponse {
    transformed_gateway_request: GatewayRequest,
}

/// Extract authorization token from headers (case-insensitive)
fn extract_auth_token(headers: &HashMap<String, String>) -> Option<&str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(AUTH_HEADER))
        .map(|(_, v)| v.strip_prefix("Bearer ").unwrap_or(v.as_str()))
}

/// Extract custom header for propagation (case-insensitive)
fn extract_custom_header(headers: &HashMap<String, String>) -> Option<&str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(CUSTOM_HEADER))
        .map(|(_, v)| v.as_str())
}

/// Extract user information from JWT token
fn extract_user_info_from_token(token: &str) -> Option<(String, String)> {
    // Split the JWT token into parts
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    
    // Decode the payload (second part)
    let payload = parts[1];
    
    // Add padding if needed
    let padded_payload = match payload.len() % 4 {
        2 => format!("{payload}=="),
        3 => format!("{payload}="),
        0 => payload.to_string(),
        _ => return None,
    };
    
    // Decode base64
    let decoded = general_purpose::URL_SAFE_NO_PAD.decode(&padded_payload).ok()?;
    let payload_str = std::str::from_utf8(&decoded).ok()?;
    
    // Parse JSON
    let claims: JwtClaims = serde_json::from_str(payload_str).ok()?;
    
    let user_id = if !claims.sub.is_empty() {
        claims.sub
    } else if !claims.preferred_username.is_empty() {
        claims.preferred_username
    } else if !claims.email.is_empty() {
        claims.email
    } else {
        return None;
    };
    
    let user_name = if claims.name.is_empty() {
        user_id.clone()
    } else {
        claims.name
    };
    
    Some((user_id, user_name))
}

async fn interceptor_handler(event: LambdaEvent<Value>) -> Result<InterceptorResponse, Error> {
    let interceptor_event: InterceptorEvent = serde_json::from_value(event.payload)?;
    let mut gateway_request = interceptor_event.mcp.gateway_request;

    // Only process tools/call requests
    let is_tool_call = gateway_request
        .body
        .as_ref()
        .and_then(|b| b.get("method"))
        .is_some_and(|m| m == "tools/call");

    if !is_tool_call {
        debug!("Skipping non-tool request");
        return Ok(InterceptorResponse {
            interceptor_output_version: "1.0".to_string(),
            mcp: McpResponse {
                transformed_gateway_request: gateway_request,
            },
        });
    }

    info!("Processing gateway request interceptor");

    // Process headers if they exist
    if let Some(ref headers) = gateway_request.headers {
        debug!("Processing {} headers", headers.len());

        // Extract tokens and headers
        let auth_token = extract_auth_token(headers);
        let custom_header = extract_custom_header(headers);

        // Modify request body to include extracted data
        if let Some(body) = &mut gateway_request.body
            && let Some(params) = body.get_mut("params")
                && let Some(args) = params.get_mut("arguments")
                && let Some(args_obj) = args.as_object_mut() {
                // Add authorization token if we have one
                if let Some(token) = auth_token {
                    args_obj.insert("authorization_token".to_string(), json!(token));
                    
                    // Extract user information from the token
                    if let Some((user_id, user_name)) = extract_user_info_from_token(token) {
                        args_obj.insert("user_id".to_string(), json!(user_id));
                        args_obj.insert("user_name".to_string(), json!(user_name));
                    }
                }
                // Add custom header if present
                if let Some(custom) = custom_header {
                    args_obj.insert(CUSTOM_HEADER.to_string(), json!(custom));
                }
            }
    }

    info!("Request processing complete");
    Ok(InterceptorResponse {
        interceptor_output_version: "1.0".to_string(),
        mcp: McpResponse {
            transformed_gateway_request: gateway_request,
        },
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Use Lambda runtime's built-in tracing subscriber for CloudWatch Logs
    lambda_runtime::tracing::init_default_subscriber();

    lambda_runtime::run(lambda_runtime::service_fn(interceptor_handler)).await
}