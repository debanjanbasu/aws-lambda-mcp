use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PersonalizedGreetingRequest {
    pub user_name: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PersonalizedGreetingResponse {
    pub greeting: String,
}
