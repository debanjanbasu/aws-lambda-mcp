//! Interceptor request/response models for AWS Bedrock `AgentCore` Gateway.
//!
//! These types define the structure of events received and responses sent
//! by the gateway interceptor Lambda function.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// AWS Bedrock `AgentCore` Gateway interceptor event structure
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InterceptorEvent {
    pub interceptor_input_version: String,
    pub mcp: McpData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct McpData {
    pub gateway_request: GatewayRequest,
}

/// Gateway request structure for interceptor response
/// Only includes headers and body as per AWS spec
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GatewayRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>
}

/// Interceptor response matching AWS Bedrock `AgentCore` Gateway specification
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InterceptorResponse {
    pub interceptor_output_version: String,
    pub mcp: McpResponse,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct McpResponse {
    pub transformed_gateway_request: GatewayRequest,
}
