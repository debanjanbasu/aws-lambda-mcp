//! Personalized greeting tool that uses user information extracted by the interceptor.

use anyhow::Result;
use rmcp::tool;
use serde::{Deserialize, Serialize};

/// Input for the personalized greeting tool.
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct PersonalizedGreetingRequest {
    /// The user's name extracted by the interceptor
    #[serde(default)]
    pub user_name: String,
    
    /// The user's ID extracted by the interceptor
    #[serde(default)]
    pub user_id: String,
    
    /// The authorization token (for demonstration purposes)
    #[serde(default)]
    pub authorization_token: String,
}

/// Output from the personalized greeting tool.
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct PersonalizedGreetingResponse {
    /// A personalized greeting message
    pub greeting: String,
    
    /// The user's name
    pub user_name: String,
    
    /// The current time
    pub timestamp: String,
}

/// Get a personalized greeting for the user.
///
/// This tool demonstrates how the interceptor can extract user information
/// from JWT tokens and pass it to the main Lambda function.
///
/// # Errors
///
/// Returns an error if there's an issue generating the greeting.
#[tool(
    description = "Get a personalized greeting for the user. This tool demonstrates how the interceptor can extract user information from JWT tokens and pass it to the main Lambda function."
)]
pub async fn get_personalized_greeting(
    request: PersonalizedGreetingRequest,
) -> Result<PersonalizedGreetingResponse> {
    // In a real implementation, you might fetch additional user data from a database
    // or external service using the user_id.
    
    let greeting = if !request.user_name.is_empty() {
        format!("Hello, {}! Welcome to our service.", request.user_name)
    } else if !request.user_id.is_empty() {
        format!("Hello! Welcome to our service. Your user ID is: {}", request.user_id)
    } else {
        "Hello! Welcome to our service.".to_string()
    };
    
    let timestamp = chrono::Utc::now().to_rfc3339();
    
    Ok(PersonalizedGreetingResponse {
        greeting,
        user_name: request.user_name,
        timestamp,
    })
}