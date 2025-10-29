use anyhow::{Result as AnyhowResult, bail};
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Deserialize)]
struct Request {
    #[serde(rename = "firstName")]
    first_name: Option<String>,
}

#[derive(Serialize)]
struct Response {
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();
    
    // Parse the request
    let request: Request = serde_json::from_value(event)?;
    
    let first_name = request.first_name.as_deref().unwrap_or("world");
    
    // Create greeting
    let response = create_greeting(first_name)?;
    
    Ok(json!({ "message": response.message }))
}

fn create_greeting(name: &str) -> AnyhowResult<Response> {
    if name.is_empty() {
        bail!("Name cannot be empty")
    }
    
    if name.len() > 100 {
        bail!("Name is too long: {} characters", name.len())
    }
    
    Ok(Response {
        message: format!("Hello, {}!", name),
    })
}