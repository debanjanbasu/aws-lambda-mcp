use anyhow::Result;
use lambda_runtime::tracing::{debug, error, info};
use lambda_runtime::{Context, Diagnostic, LambdaEvent};
use serde_json::Value;

use crate::http::{HttpClient, HTTP_CLIENT};
use crate::models::{PersonalizedGreetingRequest, WeatherRequest};
use crate::tools::{get_personalized_greeting, get_weather};
use crate::utils::strip_gateway_prefix;

/// Extracts tool name from Lambda context or MCP event payload.
///
/// Tool name resolution order:
/// 1. AWS Lambda context (Bedrock `AgentCore` Gateway)
/// 2. MCP tools/call request payload
/// 3. Default to "unknown"
///
/// # Note
///
/// According to AWS docs, tool name is passed in `context.client_context.custom[bedrockAgentCoreToolName]`.
/// For MCP, also check the event payload for tools/call method.
fn extract_tool_name(event_payload: &Value, context: &Context) -> String {
    debug!(
        "Extracting tool name from context: {:?}",
        context.client_context
    );

    // First try context (Bedrock AgentCore Gateway should set this)
    if let Some(custom) = &context.client_context
        && let Some(tool_name_value) = custom.custom.get("bedrockAgentCoreToolName")
    {
        let tool_name = tool_name_value.clone();
        debug!("Found tool name in context: {}", tool_name);
        return strip_gateway_prefix(&tool_name);
    }

    // Fallback: check if this is an MCP tools/call request
    if event_payload
        .get("method")
        .and_then(|m| m.as_str())
        .is_some_and(|method| method == "tools/call")
        && let Some(name) = event_payload
            .get("params")
            .and_then(|params| params.get("name"))
            .and_then(|n| n.as_str())
    {
        debug!("Found tool name in MCP payload: {}", name);
        return strip_gateway_prefix(name);
    }

    // Final fallback
    debug!("Tool name not found, using unknown");
    "unknown".to_string()
}

/// Routes a tool request to the appropriate handler.
///
/// Supported tools:
/// - `get_weather`: Fetches weather data for a location
/// - `get_personalized_greeting`: Generates personalized greeting for user
///
/// # Errors
///
/// Returns a `Diagnostic` error if:
/// - Tool name is not recognized (`UnknownTool`)
/// - Request payload cannot be parsed (`InvalidInput`)
/// - Tool execution fails (`ToolError`)
/// - Response cannot be serialized (`SerializationError`)
pub async fn route_tool(tool_name: &str, event_payload: Value) -> Result<Value, Diagnostic> {
    route_tool_with_client(tool_name, event_payload, &*HTTP_CLIENT).await
}

/// Routes a tool call to the appropriate handler with a custom HTTP client.
///
/// This function is primarily used for testing with mocked HTTP clients.
///
/// # Errors
///
/// This function will return a `Diagnostic` error if:
/// - The tool name is unknown
/// - Tool argument parsing fails
/// - The tool execution fails
pub async fn route_tool_with_client(tool_name: &str, event_payload: Value, http_client: &dyn HttpClient) -> Result<Value, Diagnostic> {
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

    match tool_name {
        "get_weather" => {
            let request: WeatherRequest = serde_json::from_value(tool_args).map_err(|e| {
                error!(error = %e, "Failed to parse weather request");
                Diagnostic {
                    error_type: "InvalidInput".to_string(),
                    error_message: format!("Failed to parse weather request: {e}"),
                }
            })?;

            let response = get_weather(request, http_client).await.map_err(|e| {
                error!(error = %format!("{e:#}"), "Weather tool execution failed");
                Diagnostic {
                    error_type: "ToolError".to_string(),
                    error_message: format!("{e}"),
                }
            })?;

            serde_json::to_value(response).map_err(|e| {
                error!(error = %e, "Failed to serialize weather response");
                Diagnostic {
                    error_type: "SerializationError".to_string(),
                    error_message: format!("Failed to serialize weather response: {e}"),
                }
            })
        }
        "get_personalized_greeting" => {
            let request: PersonalizedGreetingRequest = serde_json::from_value(tool_args).map_err(|e| {
                error!(error = %e, "Failed to parse personalized greeting request");
                Diagnostic {
                    error_type: "InvalidInput".to_string(),
                    error_message: format!("Failed to parse personalized greeting request: {e}"),
                }
            })?;

            let response = get_personalized_greeting(request).await.map_err(|e| {
                error!(error = %format!("{e:#}"), "Personalized greeting tool execution failed");
                Diagnostic {
                    error_type: "ToolError".to_string(),
                    error_message: format!("{e}"),
                }
            })?;

            serde_json::to_value(response).map_err(|e| {
                error!(error = %e, "Failed to serialize personalized greeting response");
                Diagnostic {
                    error_type: "SerializationError".to_string(),
                    error_message: format!("Failed to serialize personalized greeting response: {e}"),
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

/// Main Lambda event handler.
///
/// Processes incoming requests and routes them to appropriate tools.
/// Handles both AWS Lambda events and direct MCP calls.
///
/// # Event Processing
///
/// 1. Extracts tool name from context or payload
/// 2. Parses request arguments
/// 3. Routes to appropriate tool handler
/// 4. Returns JSON response or diagnostic error
///
/// # Errors
///
/// Returns a `Diagnostic` error with one of the following types:
/// - `InvalidInput`: Failed to parse the event payload into the required request type
/// - `ToolError`: The requested tool failed to execute
/// - `SerializationError`: Failed to serialize the tool response back to JSON
/// - `UnknownTool`: The requested tool name was not recognized
pub async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Diagnostic> {
    let (event_payload, context) = event.into_parts();
    let tool_name = extract_tool_name(&event_payload, &context);

    // Extract the actual payload - if it's an API Gateway event, get from body
    let payload_for_tool = event_payload
        .get("body")
        .and_then(|b| b.as_str())
        .and_then(|body_str| serde_json::from_str(body_str).ok())
        .unwrap_or(event_payload);

    info!(message = format!("Invoking tool: {}", tool_name));
    route_tool(&tool_name, payload_for_tool).await
}