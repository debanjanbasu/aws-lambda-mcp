# AWS Region Output
output "aws_region" {
  description = "AWS region where resources are deployed"
  value       = var.aws_region
}

# Project Name Output
output "project_name" {
  description = "Project name with suffix"
  value       = local.project_name_with_suffix
}

# Lambda Outputs
output "lambda_function_arn" {
  description = "ARN of the Lambda function"
  value       = aws_lambda_function.bedrock_agent_gateway.arn
}

output "lambda_function_name" {
  description = "Name of the Lambda function"
  value       = aws_lambda_function.bedrock_agent_gateway.function_name
}

output "lambda_function_version" {
  description = "Latest published version of Lambda function"
  value       = aws_lambda_function.bedrock_agent_gateway.version
}

output "lambda_alias_arn" {
  description = "ARN of the Lambda alias (disabled for cost optimization)"
  value       = null
}

output "lambda_role_arn" {
  description = "ARN of the Lambda execution role"
  value       = aws_iam_role.lambda_execution.arn
}

output "lambda_role_name" {
  description = "Name of the Lambda execution role"
  value       = aws_iam_role.lambda_execution.name
}

output "cloudwatch_log_group_name" {
  description = "Name of the CloudWatch log group"
  value       = aws_cloudwatch_log_group.lambda_logs.name
}

output "cloudwatch_log_group_arn" {
  description = "ARN of the CloudWatch log group"
  value       = aws_cloudwatch_log_group.lambda_logs.arn
}

output "agentcore_gateway_arn" {
  description = "ARN of the Amazon Bedrock AgentCore Gateway"
  value       = aws_bedrockagentcore_gateway.main.gateway_arn
}

output "agentcore_gateway_id" {
  description = "ID of the Amazon Bedrock AgentCore Gateway"
  value       = aws_bedrockagentcore_gateway.main.gateway_id
}

output "agentcore_gateway_name" {
  description = "Name of the Amazon Bedrock AgentCore Gateway"
  value       = aws_bedrockagentcore_gateway.main.name
}

output "agentcore_gateway_url" {
  description = "URL of the Amazon Bedrock AgentCore Gateway"
  value       = aws_bedrockagentcore_gateway.main.gateway_url
}

output "agentcore_gateway_target_id" {
  description = "ID of the Amazon Bedrock AgentCore Gateway Target"
  value       = aws_bedrockagentcore_gateway_target.lambda.target_id
}

# Entra ID OAuth Outputs (mandatory)
output "entra_app_client_id" {
  description = "Entra ID application client ID"
  value       = azuread_application.agentcore_app.client_id
}

output "entra_app_identifier_uri" {
  description = "Entra ID application identifier URI (for reference only - not used in scopes)"
  value       = "api://${azuread_application.agentcore_app.client_id}"
}

output "entra_app_scope" {
  description = "Entra ID application scope for user authentication"
  value       = "${azuread_application.agentcore_app.client_id}/${var.entra_oauth_scope_value}"
}

output "entra_app_scope_client_credentials" {
  description = "Entra ID application scope for client credential flows"
  value       = "${azuread_application.agentcore_app.client_id}/.default"
}

output "entra_app_scope_m365_copilot" {
  description = "Entra ID application scope for Microsoft 365 Copilot integration (same as client credentials)"
  value       = "${azuread_application.agentcore_app.client_id}/.default"
}

output "entra_app_name" {
  description = "Entra ID application name"
  value       = local.project_name_with_suffix
}

output "entra_app_object_id" {
  description = "Entra ID application object ID"
  value       = azuread_application.agentcore_app.object_id
}

output "entra_tenant_id" {
  description = "Entra ID tenant ID"
  value       = data.azuread_client_config.current.tenant_id
}

output "entra_issuer_url" {
  description = "Entra ID OpenID Connect issuer URL"
  value       = "https://login.microsoftonline.com/${data.azuread_client_config.current.tenant_id}/v2.0"
}

output "entra_discovery_url" {
  description = "Entra ID OpenID Connect discovery URL"
  value       = "https://login.microsoftonline.com/${data.azuread_client_config.current.tenant_id}/v2.0/.well-known/openid-configuration"
}

output "entra_token_url" {
  description = "Entra ID OAuth token URL"
  value       = "https://login.microsoftonline.com/${data.azuread_client_config.current.tenant_id}/oauth2/v2.0/token"
}

output "entra_client_secret" {
  description = "Entra ID client secret for OAuth 2.0 confidential clients (2 year expiry)"
  value       = tolist(azuread_application.agentcore_app.password)[0].value
  sensitive   = true
}

output "entra_client_secret_expires" {
  description = "Client secret expiration time"
  value       = tolist(azuread_application.agentcore_app.password)[0].end_date
}