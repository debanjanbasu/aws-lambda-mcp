use anyhow::{Context, Result};
use lambda_runtime::Diagnostic;
use serde_json::Value;
use tracing::{debug, error};

use crate::models::WeatherRequest;
use crate::tools::weather::get_weather;

/// Handles Lambda events and routes them to appropriate tools.
///
/// # Errors
/// Returns a `Diagnostic` error if:
/// - The event payload cannot be parsed into the expected request type
/// - The requested tool fails during execution
/// - The tool name is unknown
/// - Response serialization fails
pub async fn function_handler(
    event: lambda_runtime::LambdaEvent<Value>,
) -> Result<Value, Diagnostic> {
    let (event, context) = event.into_parts();

    // Log incoming event and context for debugging (visible only at DEBUG level)
    debug!(
        event = ?event,
        context = ?context,
        "Received Lambda event"
    );

    let tool_name = context
        .client_context
        .as_ref()
        .and_then(|cc| cc.custom.get("bedrockAgentCoreToolName"))
        .map_or("unknown", String::as_str);

    debug!(
        tool_name = %tool_name,
        "Extracted tool name from client context"
    );

    // Route to the appropriate tool based on the tool name
    if tool_name == "get_weather" {
        let request: WeatherRequest = serde_json::from_value(event).map_err(|e| {
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

        serde_json::to_value(response)
            .context("Failed to serialize response")
            .map_err(|e| Diagnostic {
                error_type: "SerializationError".to_string(),
                error_message: format!("Failed to serialize response: {e}"),
            })
    } else {
        error!(tool_name = %tool_name, "Unknown tool");
        Err(Diagnostic {
            error_type: "UnknownTool".to_string(),
            error_message: format!("Unknown tool: {tool_name}"),
        })
    }
}
