use anyhow::Result;
use aws_lambda_mcp::models::interceptor::{InterceptorEvent, InterceptorResponse, McpResponse};
use jsonwebtoken::dangerous::insecure_decode;
use lambda_runtime::{
    tracing::{debug, info, warn},
    Error, LambdaEvent, service_fn,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Minimal JWT claims for extracting user information.
#[derive(Debug, Deserialize)]
struct Claims {
    exp: Option<u64>,
    sub: Option<String>,
    name: Option<String>,
    email: Option<String>,
    preferred_username: Option<String>,
}

/// Extract authorization token from headers (case-insensitive).
fn extract_auth_token(headers: &HashMap<String, String>) -> Option<&str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
        .map(|(_, v)| v.strip_prefix("Bearer ").unwrap_or(v))
}

/// Insecurely decodes a JWT to extract user ID and name without validation.
/// Checks for token expiry.
fn extract_user_info_from_token(token: &str) -> Option<(String, String)> {
    let claims = insecure_decode::<Claims>(token).map(|d| d.claims).ok()?;

    // Check token expiry if present
    if let Some(exp) = claims.exp {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs();
        if exp < now {
            warn!(message = "Token is expired");
            return None;
        }
    }

    let user_id = claims
        .sub
        .or_else(|| claims.preferred_username.clone())
        .or(claims.email)?;

    let user_name = claims.name.unwrap_or_else(|| {
        claims
            .preferred_username
            .unwrap_or_else(|| user_id.split('@').next().unwrap_or(&user_id).to_string())
    });

    Some((user_id, user_name))
}

async fn interceptor_handler(event: LambdaEvent<Value>) -> Result<InterceptorResponse, Error> {
    let interceptor_event: InterceptorEvent = serde_json::from_value(event.payload)?;
    let mut gateway_request = interceptor_event.mcp.gateway_request;

    let is_tool_call = gateway_request
        .body
        .as_ref()
        .and_then(|b| b.get("method"))
        .is_some_and(|m| m == "tools/call");

    if !is_tool_call {
        debug!(message = "Skipping non-tool request");
        return Ok(InterceptorResponse {
            interceptor_output_version: "1.0".to_string(),
            mcp: McpResponse {
                transformed_gateway_request: gateway_request,
            },
        });
    }

    if let Some(token) = gateway_request.headers.as_ref().and_then(extract_auth_token)
        && let Some(body) = gateway_request.body.as_mut().and_then(|b| b.get_mut("params")).and_then(|p| p.get_mut("arguments")).and_then(|a| a.as_object_mut())
    {
        info!(message = "Injecting auth token into arguments");
        body.insert("auth_token".to_string(), json!(token));

        if let Some((user_id, user_name)) = extract_user_info_from_token(token) {
            info!(message = "Injecting user info into arguments");
            body.insert("user_id".to_string(), json!(user_id));
            body.insert("user_name".to_string(), json!(user_name));
        } else {
            warn!(message = "Could not extract user info from token");
        }
    }

    Ok(InterceptorResponse {
        interceptor_output_version: "1.0".to_string(),
        mcp: McpResponse {
            transformed_gateway_request: gateway_request,
        },
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::tracing::init_default_subscriber();
    lambda_runtime::run(service_fn(interceptor_handler)).await
}
