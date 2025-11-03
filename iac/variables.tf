variable "aws_region" {
  description = "AWS region to deploy resources"
  type        = string
  default     = "ap-southeast-2"
}

variable "lambda_function_name" {
  description = "Name of the Lambda function"
  type        = string
  default     = "aws-lambda-mcp"
}

variable "lambda_binary_path" {
  description = "Path to the compiled Lambda bootstrap binary"
  type        = string
  default     = "../target/lambda/aws-lambda-mcp/bootstrap"
}

variable "tool_schema_path" {
  description = "Path to the tool schema JSON file"
  type        = string
  default     = "../tool_schema.json"
}

variable "lambda_memory_size" {
  description = "Memory size for Lambda function in MB"
  type        = number
  default     = 128
}

variable "lambda_timeout" {
  description = "Timeout for Lambda function in seconds"
  type        = number
  default     = 30
}

variable "lambda_alias_name" {
  description = "Name for Lambda alias"
  type        = string
  default     = "live"
}

variable "lambda_alias_version" {
  description = "Lambda version for alias (use $LATEST for latest)"
  type        = string
  default     = "$LATEST"
}

variable "rust_log_level" {
  description = "Rust logging level (trace, debug, info, warn, error)"
  type        = string
  default     = "trace" # Reduced from 'info' to minimize CloudWatch log volume
}

variable "additional_env_vars" {
  description = "Additional environment variables for Lambda function"
  type        = map(string)
  default     = {}
}

variable "log_retention_days" {
  description = "CloudWatch Logs retention period in days"
  type        = number
  default     = 3 # Minimum for cost optimization (was 7)
}

variable "cloudwatch_kms_key_arn" {
  description = "KMS key ARN for CloudWatch Logs encryption (optional)"
  type        = string
  default     = null
}

variable "gateway_name" {
  description = "Name of the Bedrock Agent Gateway"
  type        = string
  default     = "aws-lambda-mcp-gateway"
}

variable "secrets_manager_arns" {
  description = "List of Secrets Manager secret ARNs the Lambda can access"
  type        = list(string)
  default     = []
}

variable "common_tags" {
  description = "Common tags to apply to all resources"
  type        = map(string)
  default = {
    Project     = "aws-lambda-mcp"
    ManagedBy   = "terraform"
    Environment = "production"
  }
}

# Entra ID OAuth Configuration
variable "entra_app_name" {
  description = "Entra ID application registration name"
  type        = string
  default     = "aws-lambda-mcp"
}

variable "entra_sign_in_audience" {
  description = "Entra ID sign-in audience (AzureADMyOrg, AzureADMultipleOrgs, AzureADandPersonalMicrosoftAccount)"
  type        = string
  default     = "AzureADMyOrg"
}

variable "entra_redirect_uris" {
  description = "List of redirect URIs for OAuth callbacks"
  type        = list(string)
  default     = ["http://localhost:6274/callback/"]
}

variable "entra_group_membership_claims" {
  description = "Group membership claims to include in tokens"
  type        = list(string)
  default     = ["SecurityGroup"]
}

variable "entra_oauth_scope_value" {
  description = "OAuth scope value (used in token requests)"
  type        = string
  default     = "access_as_user"
}

variable "entra_oauth_scope_admin_name" {
  description = "OAuth scope admin consent display name"
  type        = string
  default     = "Access Bedrock Gateway"
}

variable "entra_oauth_scope_admin_description" {
  description = "OAuth scope admin consent description"
  type        = string
  default     = "Allow the application to access the Bedrock Gateway on behalf of the signed-in user"
}

variable "entra_oauth_scope_user_name" {
  description = "OAuth scope user consent display name"
  type        = string
  default     = "Access Bedrock Gateway"
}

variable "entra_oauth_scope_user_description" {
  description = "OAuth scope user consent description"
  type        = string
  default     = "Allow the application to access the Bedrock Gateway on your behalf"
}

variable "entra_app_tags" {
  description = "Tags for Entra ID application"
  type        = list(string)
  default     = ["bedrock-gateway", "oauth2", "pkce", "terraform-managed"]
}
