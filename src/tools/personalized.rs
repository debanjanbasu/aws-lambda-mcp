use crate::models::error::AppError;
use crate::models::personalized::{PersonalizedGreetingRequest, PersonalizedGreetingResponse};
use anyhow::Result;

/// Default name to use when no user information is available
const DEFAULT_USER_NAME: &str = "there";

/// Generates a personalized greeting for a user.
///
/// This tool creates friendly greetings using user information injected by the interceptor:
/// - Uses `user_name` if provided
/// - Extracts name from `user_id` (email) if available
/// - Defaults to "there" if no user information is available
///
/// # Examples
///
/// With `user_name`: "Hello, John!"
/// With `user_id`: "Hello, jane.doe!"
/// Without user info: "Hello, there!"
///
/// # Errors
///
/// This function currently does not return errors but uses `Result` for API consistency.
/// Future enhancements may add error conditions.
pub async fn get_personalized_greeting(
    request: PersonalizedGreetingRequest,
) -> Result<PersonalizedGreetingResponse, AppError> {
    let user_name = extract_user_name(&request);
    let greeting = format!("Hello, {user_name}!");
    Ok(PersonalizedGreetingResponse { greeting })
}

/// Extracts a user name from the request
fn extract_user_name(request: &PersonalizedGreetingRequest) -> String {
    if !request.user_name.is_empty() {
        return request.user_name.clone();
    }
    
    if !request.user_id.is_empty() {
        // Extract user name from user ID (email) if available
        return request.user_id
            .split('@')
            .next()
            .unwrap_or(DEFAULT_USER_NAME)
            .to_string();
    }
    
    DEFAULT_USER_NAME.to_string()
}
