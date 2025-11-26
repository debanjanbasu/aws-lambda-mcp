use lambda_runtime::{Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, info, instrument};

// Header key constants for maintainability
const AUTH_HEADER: &str = "authorization";
const CUSTOM_HEADER: &str = "customHeaderKey";

/// Simplified interceptor event structure - only handles REQUEST interception
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InterceptorEvent {
    gateway_request: GatewayRequest,
}

/// Gateway request structure
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct GatewayRequest {
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
}

/// Simplified interceptor response - only transforms requests
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InterceptorResponse {
    version: String,
    transformed_gateway_request: GatewayRequest,
}

/// Extract authorization token from headers
fn extract_auth_header(headers: &HashMap<String, String>) -> Option<String> {
    headers.get(AUTH_HEADER).cloned()
}

/// Extract custom header for propagation
fn extract_custom_header(headers: &HashMap<String, String>) -> Option<String> {
    headers.get(CUSTOM_HEADER).cloned()
}

/// Perform token exchange (placeholder)
fn exchange_token(_auth_token: &str) -> String {
    "exchanged_token_placeholder".to_string()
}

#[instrument(skip(event))]
async fn interceptor_handler(event: LambdaEvent<InterceptorEvent>) -> Result<InterceptorResponse, Error> {
    let payload = event.payload;
    let mut gateway_request = payload.gateway_request;

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
    Ok(InterceptorResponse {
        version: "1.0".to_string(),
        transformed_gateway_request: gateway_request,
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configure tracing for CloudWatch Logs
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .init();

    info!("Starting Bedrock AgentCore Gateway interceptor");

    lambda_runtime::run(lambda_runtime::service_fn(interceptor_handler)).await
}