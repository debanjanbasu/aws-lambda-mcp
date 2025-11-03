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

output "bedrock_gateway_arn" {
  description = "ARN of the Bedrock Agent Gateway"
  value       = aws_bedrockagentcore_gateway.main.gateway_arn
}

output "bedrock_gateway_id" {
  description = "ID of the Bedrock Agent Gateway"
  value       = aws_bedrockagentcore_gateway.main.gateway_id
}

output "bedrock_gateway_name" {
  description = "Name of the Bedrock Agent Gateway"
  value       = aws_bedrockagentcore_gateway.main.gateway_name
}

output "bedrock_gateway_target_id" {
  description = "ID of the Bedrock Agent Gateway Target"
  value       = aws_bedrockagentcore_gateway_target.lambda.id
}

# Entra ID OAuth Outputs (mandatory)
output "entra_app_client_id" {
  description = "Entra ID application client ID"
  value       = azuread_application.bedrock_gateway.client_id
}

output "entra_app_object_id" {
  description = "Entra ID application object ID"
  value       = azuread_application.bedrock_gateway.object_id
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
