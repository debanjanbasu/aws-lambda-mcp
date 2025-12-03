use crate::models::personalized::{PersonalizedGreetingRequest, PersonalizedGreetingResponse};
use anyhow::Result;

/// Generates a personalized greeting for a user.
///
/// # Errors
///
/// This function will currently not return an error, but is designed to return a `Result`
/// to conform to the tool interface and allow for future error handling if needed.
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
