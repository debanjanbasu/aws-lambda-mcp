use anyhow::Result;
use crate::models::personalized::{PersonalizedGreetingRequest, PersonalizedGreetingResponse};

/// Generates a personalized greeting for a user.
///
/// # Errors
///
/// This function will currently not return an error, but is designed to return a `Result`
/// to conform to the tool interface and allow for future error handling if needed.
pub async fn get_personalized_greeting(
    request: PersonalizedGreetingRequest,
) -> Result<PersonalizedGreetingResponse> {
    let user_name = request.user_name;
    let greeting = format!("Hello, {user_name}!");
    Ok(PersonalizedGreetingResponse { greeting })
}
