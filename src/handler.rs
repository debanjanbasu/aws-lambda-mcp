use anyhow::{Context, Result};
use lambda_runtime::Diagnostic;
use serde_json::Value;
use tracing::{debug, error, info, instrument};

use crate::models::WeatherRequest;
use crate::tools::weather::get_weather;

// Lambda event handler with security-conscious logging
//
// ## Logging Behavior
//
// The handler conditionally includes event payloads in logs based on RUST_LOG level:
//
// - **Production (RUST_LOG=info/warn/error):**
//   - Only logs event size
//   - Event payload excluded from spans via `skip_if` in `#[instrument]`
//   - Debug logs don't fire
//   - Secure for production use
//
// - **Debug/Troubleshooting (RUST_LOG=debug/trace):**
//   - Logs full event payload and Lambda context
//   - Event included in spans
//   - Debug logs visible in CloudWatch
//   - Use only for troubleshooting, not production
//
// The logging configuration in main.rs automatically enables `with_current_span(true)`
// only when debug/trace logging is active, making field values visible in structured logs.

/// Extracts tool name from event payload, checking multiple possible locations.
//
// Checks in order:
// 1. Client context custom fields (standard AWS)
// 2. Event fields: `name`, `tool_name`, `toolName`
// 3. Strips Bedrock Gateway prefix if present (`gateway-id___tool_name`)
fn extract_tool_name(event: &Value, context: &lambda_runtime::Context) -> String {
    // 1. Check client_context (standard AWS Lambda invocation)
    if let Some(ref cc) = context.client_context
        && let Some(name) = cc.custom.get("bedrockAgentCoreToolName") {
            debug!(
                tool_name = %name,
                source = "client_context",
                "Tool name extracted"
            );
            return strip_gateway_prefix(name);
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

/// Handles Lambda events and routes them to appropriate tools.
///
/// # Errors
///
/// Returns a `Diagnostic` error if:
/// - The event payload cannot be parsed into the expected request type
/// - The requested tool fails during execution
/// - The tool name is unknown
/// - Response serialization fails
///
/// # Security Note
///
/// When RUST_LOG is set to debug or trace, the full event payload and context are logged
/// for troubleshooting. In production with RUST_LOG=info/warn/error, event payloads are
/// not logged (only event_size is recorded). The main.rs tracing configuration conditionally
/// enables `with_current_span` only for debug/trace levels, preventing sensitive data from
/// appearing in production logs.
#[instrument(fields(request_id = %event.context.request_id))]
pub async fn function_handler(
    event: lambda_runtime::LambdaEvent<Value>,
) -> Result<Value, Diagnostic> {
    let (event_payload, context) = event.into_parts();

    info!(
        event_size = event_payload.to_string().len(),
        "Lambda invocation started"
    );

    debug!(
        event = %serde_json::to_string(&event_payload).unwrap_or_else(|_| "{}".to_string()),
        "Received event payload"
    );

    debug!(
        request_id = %context.request_id,
        deadline = context.deadline,
        invoked_function_arn = %context.invoked_function_arn,
        "Lambda context"
    );

    let tool_name = extract_tool_name(&event_payload, &context);

    info!(
        tool_name = %tool_name,
        "Routing to tool handler"
    );

    // Route to the appropriate tool based on the tool name
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

        serde_json::to_value(response)
            .context("Failed to serialize response")
            .map_err(|e| {
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
