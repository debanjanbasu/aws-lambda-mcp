use anyhow::Result;
use lambda_runtime::tracing::{debug, error, info};
use lambda_runtime::{Context, Diagnostic, LambdaEvent};
use serde_json::Value;

use crate::models::{PersonalizedGreetingRequest, WeatherRequest};
use crate::tools::{get_personalized_greeting, get_weather};

/// Extracts tool name from Lambda context or MCP event payload.
/// According to AWS docs, tool name is passed in `context.client_context.custom[bedrockAgentCoreToolName]`.
/// For MCP, also check the event payload for tools/call method.
fn extract_tool_name(event_payload: &Value, context: &Context) -> String {
    debug!(
        "Extracting tool name from context: {:?}",
        context.client_context
    );

    // First try context (Bedrock AgentCore Gateway should set this)
    if let Some(tool_name) = context
        .client_context
        .as_ref()
        .and_then(|cc| cc.custom.get("bedrockAgentCoreToolName"))
        .map(String::as_str)
    {
        debug!("Found tool name in context: {}", tool_name);
        return strip_gateway_prefix(tool_name);
    }

    // Fallback: check if this is an MCP tools/call request
    if let Some(method) = event_payload.get("method").and_then(|m| m.as_str())
        && method == "tools/call"
        && let Some(params) = event_payload.get("params")
        && let Some(name) = params.get("name").and_then(|n| n.as_str())
    {
        debug!("Found tool name in MCP payload: {}", name);
        return name.to_string();
    }

    // Final fallback
    debug!("Tool name not found, using unknown");
    "unknown".to_string()
}

// Strips Bedrock Gateway prefix from tool name.
//
// Format: `gateway-target-id___tool_name` → `tool_name`
pub fn strip_gateway_prefix(name: &str) -> String {
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
///
/// # Errors
///
/// Returns a `Diagnostic` error if the tool is unknown or if tool execution fails.
pub async fn route_tool(tool_name: &str, event_payload: Value) -> Result<Value, Diagnostic> {
    debug!(tool_name = %tool_name, "Entering route_tool function");
    debug!(
        "Routing tool: {} with payload: {:?}",
        tool_name, event_payload
    );

    // Extract arguments from MCP request structure if present
    let tool_args = event_payload
        .get("params")
        .and_then(|params| params.get("arguments"))
        .unwrap_or(&event_payload)
        .clone();

    debug!("Extracted tool arguments: {:?}", tool_args);

    macro_rules! handle_tool {
        ($tool_fn:expr, $request_type:ty, $tool_args:expr) => {{
            let request: $request_type =
                serde_json::from_value($tool_args).map_err(|e| {
                    error!(error = %e, "Failed to parse request");
                    Diagnostic {
                        error_type: "InvalidInput".to_string(),
                        error_message: format!("Failed to parse request: {e}"),
                    }
                })?;

            let response = $tool_fn(request).await.map_err(|e| {
                // Use {:#} to get the full error chain with causes
                error!(error = %format!("{e:#}"), "Tool execution failed");
                Diagnostic {
                    error_type: "ToolError".to_string(),
                    error_message: format!("{e:#}"),
                }
            })?;

            serde_json::to_value(response).map_err(|e| {
                error!(error = %e, "Failed to serialize response");
                Diagnostic {
                    error_type: "SerializationError".to_string(),
                    error_message: format!("Failed to serialize response: {e}"),
                }
            })
        }};
    }

    match tool_name {
        "get_weather" => handle_tool!(get_weather, WeatherRequest, tool_args),
        "get_personalized_greeting" => {
            handle_tool!(
                get_personalized_greeting,
                PersonalizedGreetingRequest,
                tool_args
            )
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

/// Lambda event handler. Routes to tools based on event.name or `client_context.custom` fields.
/// Logs full event when `RUST_LOG=debug/trace`, only `event_size` in production.
///
/// # Errors
///
/// Returns a `Diagnostic` error with one of the following types:
///
/// - `InvalidInput`: Failed to parse the event payload into the required request type
/// - `ToolError`: The requested tool failed to execute
/// - `SerializationError`: Failed to serialize the tool response back to JSON
/// - `UnknownTool`: The requested tool name was not recognized
pub async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Diagnostic> {
    let (event_payload, context) = event.into_parts();

    let tool_name = extract_tool_name(&event_payload, &context);

    // Extract the actual payload - if it's an API Gateway event, get from body
    let payload_for_tool =
        if let Some(body_str) = event_payload.get("body").and_then(|b| b.as_str()) {
            serde_json::from_str(body_str).unwrap_or(event_payload)
        } else {
            event_payload
        };

    info!(message = format!("Invoking tool: {}", tool_name));

    route_tool(&tool_name, payload_for_tool).await
}
