use anyhow::Result;
use lambda_runtime::{Context, Diagnostic, LambdaEvent};
use serde_json::Value;
use tracing::{debug, error, info, instrument};

use crate::models::WeatherRequest;
use crate::tools::weather::get_weather;

// Lambda event handler with conditional debug logging.
// - Production (RUST_LOG=info/warn/error): Only event_size logged, event skipped from spans
// - Debug (RUST_LOG=debug/trace): Full event + context logged via println!

/// Extracts tool name from event, checking `client_context.custom` or event.name fields.
/// Strips Bedrock Gateway prefix if present.
fn extract_tool_name(event_payload: &Value, context: &Context) -> String {
    let tool_name = context
        .client_context
        .as_ref()
        .and_then(|cc| cc.custom.get("bedrockAgentCoreToolName"))
        .map_or_else(
            || event_payload["name"].as_str().unwrap_or("unknown"),
            String::as_str,
        );

    strip_gateway_prefix(tool_name)
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

/// Routes a tool request to the appropriate handler.
async fn route_tool(tool_name: &str, event_payload: Value) -> Result<Value, Diagnostic> {
    if tool_name == "get_weather" {
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

/// Lambda event handler. Routes to tools based on event.name or `client_context.custom` fields.
/// Logs full event when `RUST_LOG=debug/trace`, onl`event_size`ze in production.
///
/// # Errors
///
/// Returns a `Diagnostic` error with one of the following types:
///
/// - `InvalidInput`: Failed to parse the event payload into the required request type
/// - `ToolError`: The requested tool failed to execute
/// - `SerializationError`: Failed to serialize the tool response back to JSON
/// - `UnknownTool`: The requested tool name was not recognized
#[instrument(skip(event), fields(req_id = %event.context.request_id))]
pub async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Diagnostic> {
    // No point in trying to log event or context, they're obfuscated for privacy
    let (event_payload, context) = event.into_parts();

    info!(
        event_size = event_payload.to_string().len(),
        "Lambda invocation started"
    );

    let tool_name = extract_tool_name(&event_payload, &context);
    info!(tool_name = %tool_name, "Routing to tool handler");

    route_tool(&tool_name, event_payload).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_strip_gateway_prefix() {
        assert_eq!(
            strip_gateway_prefix("gateway-123___get_weather"),
            "get_weather"
        );
        assert_eq!(strip_gateway_prefix("get_weather"), "get_weather");
        assert_eq!(strip_gateway_prefix(""), "");
        assert_eq!(strip_gateway_prefix("no_prefix"), "no_prefix");
        assert_eq!(
            strip_gateway_prefix("complex___prefix___tool_name"),
            "prefix___tool_name"
        );
    }

    #[tokio::test]
    async fn test_route_tool_unknown_tool() {
        let event_payload = json!({ "name": "unknown_tool" });
        let result = route_tool("unknown_tool", event_payload).await;
        assert!(result.is_err());

        let diagnostic = result.unwrap_err();
        assert_eq!(diagnostic.error_type, "UnknownTool");
        assert!(diagnostic.error_message.contains("Unknown tool: unknown_tool"));
    }

    #[tokio::test]
    async fn test_route_tool_invalid_weather_request() {
        let event_payload = json!({ "invalid_field": "value" }); // Missing location field
        let result = route_tool("get_weather", event_payload).await;
        assert!(result.is_err());

        let diagnostic = result.unwrap_err();
        assert_eq!(diagnostic.error_type, "InvalidInput");
        assert!(diagnostic.error_message.contains("Failed to parse request"));
    }

    // Note: Testing successful weather routing would require mocking the get_weather function
    // which is complex in this context. The integration tests cover the full flow.
}
