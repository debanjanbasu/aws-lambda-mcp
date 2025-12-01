variable "aws_region" {
  description = "AWS region to deploy resources"
  type        = string
  default     = "ap-southeast-2"
}

variable "project_name" {
  description = "Project name (used for Lambda function, gateway, and Entra app)"
  type        = string
  default     = "aws-agentcore-gateway"
}

variable "project_name_suffix" {
  description = "Optional suffix for project name (auto-generated if not provided)"
  type        = string
  default     = ""
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

variable "rust_log_level" {
  description = "Rust logging level. debug/trace logs full event payloads (use for troubleshooting). info/warn/error logs only event size (production setting for security)"
  type        = string
  default     = "info"

  validation {
    condition     = contains(["trace", "debug", "info", "warn", "error"], var.rust_log_level)
    error_message = "Rust log level must be one of: trace, debug, info, warn, error."
  }
}

variable "additional_env_vars" {
  description = "Additional environment variables for Lambda function"
  type        = map(string)
  default     = {}
}

variable "lambda_concurrent_executions" {
  description = "Reserved concurrent executions for main Lambda function"
  type        = number
  default     = 100  # Base concurrency - interceptor gets 2x this value

  validation {
    condition     = var.lambda_concurrent_executions > 0 && var.lambda_concurrent_executions <= 1000
    error_message = "Lambda concurrent executions must be between 1 and 1000."
  }
}

variable "log_retention_days" {
  description = "CloudWatch Logs retention period in days"
  type        = number
  default     = 3  # 3 days retention for cost optimization
}



variable "common_tags" {
  description = "Common tags to apply to all resources"
  type        = map(string)
  default = {
    Project     = "aws-agentcore-gateway"
    ManagedBy   = "terraform"
    Environment = "production"
  }
}

# Entra ID OAuth Configuration
variable "entra_sign_in_audience" {
  description = "Entra ID sign-in audience"
  type        = string
  default     = "AzureADMultipleOrgs"
}

variable "entra_redirect_uris" {
  description = "List of redirect URIs for OAuth callbacks"
  type        = list(string)
  default     = []
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

# Amazon Bedrock AgentCore Gateway Configuration
variable "gateway_exception_level" {
  description = "Exception level for Gateway error logging. Valid values are DEBUG, INFO, WARN, ERROR, or null for disabled."
  type        = string
  default     = null

  validation {
    condition     = var.gateway_exception_level == null || contains(["DEBUG", "INFO", "WARN", "ERROR"], var.gateway_exception_level)
    error_message = "Exception level must be one of: DEBUG, INFO, WARN, ERROR, or null."
  }
}


