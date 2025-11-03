use anyhow::{Context, Result};
use lambda_runtime::Diagnostic;
use serde_json::Value;
use tracing::{debug, error, info, instrument};

use crate::models::WeatherRequest;
use crate::tools::weather::get_weather;

/// Extracts tool name from event payload, checking multiple possible locations.
///
/// Checks in order:
/// 1. Client context custom fields (standard AWS)
/// 2. Event fields: `name`, `tool_name`, `toolName`
/// 3. Strips Bedrock Gateway prefix if present (`gateway-id___tool_name`)
fn extract_tool_name(event: &Value, context: &lambda_runtime::Context) -> String {
    // 1. Check client_context (standard AWS Lambda invocation)
    if let Some(ref cc) = context.client_context {
        if let Some(name) = cc.custom.get("bedrockAgentCoreToolName") {
            debug!(
                tool_name = %name,
                source = "client_context",
                "Tool name extracted"
            );
            return strip_gateway_prefix(name);
        }
    }

    // 2. Check event payload (Bedrock Agent Core Gateway format)
    event
        .get("name")
        .or_else(|| event.get("tool_name"))
        .or_else(|| event.get("toolName"))
        .and_then(|v| v.as_str())
        .map_or_else(
            || {
                debug!("No tool name found in event");
                "unknown".to_string()
            },
            |name| {
                debug!(
                    tool_name = %name,
                    source = "event_payload",
                    "Tool name extracted"
                );
                strip_gateway_prefix(name)
            },
        )
}

/// Strips Bedrock Gateway prefix from tool name.
///
/// Format: `gateway-target-id___tool_name` → `tool_name`
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

/// Handles Lambda events and routes them to appropriate tools.
///
/// # Errors
/// Returns a `Diagnostic` error if:
/// - The event payload cannot be parsed into the expected request type
/// - The requested tool fails during execution
/// - The tool name is unknown
/// - Response serialization fails
#[instrument(skip(event), fields(request_id = %event.context.request_id))]
pub async fn function_handler(
    event: lambda_runtime::LambdaEvent<Value>,
) -> Result<Value, Diagnostic> {
    let (event_payload, context) = event.into_parts();

    info!(
        event_size = event_payload.to_string().len(),
        "Lambda invocation started"
    );

    debug!(
        event = %serde_json::to_string(&event_payload).unwrap_or_default(),
        "Received event payload"
    );

    let tool_name = extract_tool_name(&event_payload, &context);

    info!(
        tool_name = %tool_name,
        "Routing to tool handler"
    );

    // Route to the appropriate tool based on the tool name
    match tool_name.as_str() {
        "get_weather" => {
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

            serde_json::to_value(response)
                .context("Failed to serialize response")
                .map_err(|e| {
                    error!(error = %e, "Failed to serialize response");
                    Diagnostic {
                        error_type: "SerializationError".to_string(),
                        error_message: format!("Failed to serialize response: {e}"),
                    }
                })
        }
        _ => {
            error!(tool_name = %tool_name, "Unknown tool requested");
            Err(Diagnostic {
                error_type: "UnknownTool".to_string(),
                error_message: format!("Unknown tool: {tool_name}"),
            })
        }
    }
}
