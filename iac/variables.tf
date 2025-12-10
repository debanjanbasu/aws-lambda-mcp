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
  default     = 256 # Increased from 128MB to handle ICU data initialization
}

variable "lambda_timeout" {
  description = "Timeout for Lambda function in seconds"
  type        = number
  default     = 60 # Increased from 30s for better reliability with external API calls
}

variable "rust_log_level" {
  description = "Rust logging level. debug/trace logs full event payloads (use for troubleshooting). info/warn/error logs only event size (production setting for security)"
  type        = string
  default     = "trace"

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

variable "log_retention_days" {
  description = "CloudWatch Logs retention period in days"
  type        = number
  default     = 3 # 3 days retention for cost optimization
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

# ===================================================================
# Entra App Metadata - Governance & Discovery Tags
# ===================================================================

# Ownership & Business
variable "entra_app_owner" {
  description = "Primary accountable team/email for the Entra application"
  type        = string
  default     = ""
}

variable "entra_business_unit" {
  description = "Business unit/department for chargeback"
  type        = string
  default     = ""
}

variable "entra_app_name" {
  description = "Canonical workload/app name (defaults to project_name if empty)"
  type        = string
  default     = ""
}

# Environment & Security
variable "entra_environment" {
  description = "Environment: dev|test|stage|prod"
  type        = string
  default     = "dev"

  validation {
    condition     = contains(["dev", "test", "stage", "prod"], var.entra_environment)
    error_message = "entra_environment must be one of: dev, test, stage, prod"
  }
}

variable "entra_data_classification" {
  description = "Data classification: Public|Internal|Confidential|Restricted"
  type        = string
  default     = "Internal"

  validation {
    condition     = contains(["Public", "Internal", "Confidential", "Restricted"], var.entra_data_classification)
    error_message = "entra_data_classification must be one of: Public, Internal, Confidential, Restricted"
  }
}

variable "entra_pii_processing" {
  description = "PII processing level: None|Minimal|High"
  type        = string
  default     = "None"

  validation {
    condition     = contains(["None", "Minimal", "High"], var.entra_pii_processing)
    error_message = "entra_pii_processing must be one of: None, Minimal, High"
  }
}

# Lifecycle (dates auto-computed if empty: goLiveDate=today, retireBy=+30days)
variable "entra_go_live_date" {
  description = "Go-live date (ISO 8601), e.g., 2025-12-09. Defaults to today if empty."
  type        = string
  default     = ""
}

variable "entra_retire_by_date" {
  description = "Retire-by date (ISO 8601), e.g., 2026-01-08. Defaults to 30 days from today if empty."
  type        = string
  default     = ""
}

# Operational
variable "entra_app_version" {
  description = "Application configuration version"
  type        = string
  default     = "1.0.0"
}

variable "entra_mcp_tools_override" {
  description = "Override MCP tools (auto-detected from tool_schema.json if empty)"
  type        = list(string)
  default     = []
}

variable "entra_secrets_expiry_date" {
  description = "Next secret renewal date (ISO 8601) or 'MI-only' (auto-computed from password expiry if empty)"
  type        = string
  default     = ""
}

variable "entra_graph_scopes_summary" {
  description = "Summarized Graph scopes (semicolon separated), e.g., 'openid;profile;email'"
  type        = string
  default     = "openid;profile;email"
}

# Discovery
variable "entra_created_in_tool" {
  description = "Where authored: CopilotStudio|Foundry|AgentToolkit"
  type        = string
  default     = "AgentToolkit"

  validation {
    condition     = contains(["CopilotStudio", "Foundry", "AgentToolkit"], var.entra_created_in_tool)
    error_message = "entra_created_in_tool must be one of: CopilotStudio, Foundry, AgentToolkit"
  }
}

variable "entra_created_by_user" {
  description = "Maker's UPN or service principal ID (auto-detected from Azure context if empty)"
  type        = string
  default     = ""
}

variable "entra_installed_by_count" {
  description = "Number of users who installed the agent"
  type        = number
  default     = 0
}

# Amazon Bedrock AgentCore Gateway Configuration
variable "gateway_exception_level" {
  description = "Exception level for Gateway error logging (only DEBUG is supported by Bedrock AgentCore)"
  type        = string
  default     = "DEBUG"

  validation {
    condition     = var.gateway_exception_level == "DEBUG"
    error_message = "Bedrock AgentCore Gateway only supports DEBUG as exception level."
  }
}


