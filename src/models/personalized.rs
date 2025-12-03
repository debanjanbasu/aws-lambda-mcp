use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PersonalizedGreetingRequest {
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub user_name: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PersonalizedGreetingResponse {
    pub greeting: String,
}
