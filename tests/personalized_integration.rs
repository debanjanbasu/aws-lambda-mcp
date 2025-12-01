//! Integration tests for personalized tools

use aws_lambda_mcp::tools::personalized::{get_personalized_greeting, PersonalizedGreetingRequest};

#[tokio::test]
async fn test_get_personalized_greeting() {
    let request = PersonalizedGreetingRequest {
        user_name: "John Doe".to_string(),
        user_id: "user123".to_string(),
        authorization_token: "fake_token".to_string(),
    };

    let result = get_personalized_greeting(request).await;
    assert!(result.is_ok());

    if let Ok(response) = result {
        assert!(response.greeting.contains("John Doe"));
        assert_eq!(response.user_name, "John Doe");
        assert!(!response.timestamp.is_empty());
    }
}

#[tokio::test]
async fn test_get_personalized_greeting_no_name() {
    let request = PersonalizedGreetingRequest {
        user_name: String::new(),
        user_id: "user123".to_string(),
        authorization_token: "fake_token".to_string(),
    };

    let result = get_personalized_greeting(request).await;
    assert!(result.is_ok());

    if let Ok(response) = result {
        assert!(response.greeting.contains("user123"));
        assert_eq!(response.user_name, "");
        assert!(!response.timestamp.is_empty());
    }
}