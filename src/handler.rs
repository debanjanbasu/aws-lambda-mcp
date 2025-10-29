use anyhow::{bail, Result as AnyhowResult};
use crate::models::{Request, Response};
use serde_json::{from_value, json, Value};
use lambda_runtime::Diagnostic;
use tracing::debug;

pub async fn function_handler(event: lambda_runtime::LambdaEvent<Value>) -> Result<Value, Diagnostic> {
    let (event, context) = event.into_parts();
    
    // Log incoming event for debugging (visible only at DEBUG level)
    debug!(request_id = %context.request_id, event = ?event, "Received Lambda event");
    
    // Parse request and extract name with default fallback
    let request: Request = from_value(event).map_err(anyhow::Error::from)?;
    let name = request.first_name.as_deref().unwrap_or("world");
    
    // Log parsed input for traceability
    debug!(request_id = %context.request_id, input_name = ?name, "Processing greeting request");
    
    // Generate response and serialize to JSON
    let response = create_greeting(name)?;
    Ok(json!({ "message": response.message }))
}

fn create_greeting(name: &str) -> AnyhowResult<Response> {
    match name.len() {
        0 => bail!("Name cannot be empty"),
        l if l > 100 => bail!("Name is too long: {l} characters"),
        _ => Ok(Response {
            message: format!("Hello, {name}!"),
        }),
    }
}