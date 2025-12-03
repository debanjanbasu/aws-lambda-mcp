use crate::models::personalized::{PersonalizedGreetingRequest, PersonalizedGreetingResponse};
use anyhow::Result;

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
) -> Result<PersonalizedGreetingResponse> {
    let user_name = if !request.user_name.is_empty() {
        request.user_name
    } else if !request.user_id.is_empty() {
        // Extract user name from user ID (email) if available
        request
            .user_id
            .split('@')
            .next()
            .unwrap_or("there")
            .to_string()
    } else {
        "there".to_string()
    };

    let greeting = format!("Hello, {user_name}!");
    Ok(PersonalizedGreetingResponse { greeting })
}
