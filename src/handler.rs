use anyhow::{Context, Result};
use lambda_runtime::Diagnostic;
use serde_json::Value;
use tracing::{debug, error, info, instrument};

use crate::models::WeatherRequest;
use crate::tools::weather::get_weather;

// Lambda event handler with conditional debug logging.
// - Production (RUST_LOG=info/warn/error): Only event_size logged, event skipped from spans
// - Debug (RUST_LOG=debug/trace): Full event + context logged via println!

/// Extracts tool name from event, checking client_context.custom or event.name fields.
/// Strips Bedrock Gateway prefix if present.
fn extract_tool_name(event: &Value, context: &lambda_runtime::Context) -> String {
    // Check client_context custom fields first (standard AWS invocation)
    if let Some(ref cc) = context.client_context
        && let Some(name) = cc.custom.get("bedrockAgentCoreToolName") {
            debug!(tool_name = %name, source = "client_context");
            return strip_gateway_prefix(name);
        }

    // Check event payload fields
    event
        .get("name")
        .or_else(|| event.get("tool_name"))
        .or_else(|| event.get("toolName"))
        .and_then(|v| v.as_str())
        .map_or_else(
            || {
                debug!("No tool name found");
                "unknown".to_string()
            },
            |name| {
                debug!(tool_name = %name, source = "event_payload");
                strip_gateway_prefix(name)
            },
        )
}

// Strips Bedrock Gateway prefix from tool name.
//
// Format: `gateway-target-id___tool_name` → `tool_name`
fn strip_gateway_prefix(name: &str) -> String {
    if let Some((_, actual_name)) = name.split_once("___") {
        debug!(
            original = %name,
            stripped = %actual_name,
            "Stripped Gateway prefix"
        );
        actual_name.to_string()
    } else {
        name.to_string()
    }
}

/// Lambda event handler. Routes to tools based on event.name or client_context.custom fields.
/// Logs full event when RUST_LOG=debug/trace, only event_size in production.
#[instrument(skip(event), fields(req_id = %event.context.request_id))]
pub async fn function_handler(
    event: lambda_runtime::LambdaEvent<Value>,
) -> Result<Value, Diagnostic> {
    let (event_payload, context) = event.into_parts();

    info!(event_size = event_payload.to_string().len(), "Lambda invocation started");

    // Debug: log full event (println! bypasses tracing field serialization issues)
    if tracing::level_filters::LevelFilter::current() >= tracing::Level::DEBUG {
        println!("DEBUG: Event={}", serde_json::to_string(&event_payload).unwrap_or_default());
        println!("DEBUG: Context req_id={} deadline={} arn={} mem={}MB", 
            context.request_id, context.deadline, context.invoked_function_arn, context.env_config.memory);
        if let Some(ref cc) = context.client_context {
            println!("DEBUG: ClientContext={}", serde_json::to_string(cc).unwrap_or_default());
        }
    }

    let tool_name = extract_tool_name(&event_payload, &context);

    info!(tool_name = %tool_name, "Routing to tool handler");

    if tool_name.as_str() == "get_weather" {
        let request: WeatherRequest = serde_json::from_value(event_payload).map_err(|e| {
            error!(error = %e, "Failed to parse WeatherRequest");
            Diagnostic {
                error_type: "InvalidInput".to_string(),
                error_message: format!("Failed to parse request: {e}"),
            }
        })?;

        let response = get_weather(request).await.map_err(|e| {
            error!(error = %e, "Failed to get weather");
            Diagnostic {
                error_type: "ToolError".to_string(),
                error_message: format!("Failed to get weather: {e}"),
            }
        })?;

        info!("Weather data retrieved successfully");

        serde_json::to_value(response).map_err(|e| {
            error!(error = %e, "Failed to serialize response");
            Diagnostic {
                error_type: "SerializationError".to_string(),
                error_message: format!("Failed to serialize response: {e}"),
            }
        })
    } else {
        error!(tool_name = %tool_name, "Unknown tool requested");
        Err(Diagnostic {
            error_type: "UnknownTool".to_string(),
            error_message: format!("Unknown tool: {tool_name}"),
        })
    }
}
