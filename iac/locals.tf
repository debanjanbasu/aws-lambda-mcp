# Local values - centralized configuration

locals {
  # Project naming with optional suffix
  project_name_with_suffix = var.project_name_suffix == null ? "${var.project_name}-${random_string.project_suffix.result}" : (var.project_name_suffix != "" ? "${var.project_name}-${var.project_name_suffix}" : var.project_name)

  # Microsoft Graph API identifiers (well-known GUIDs)
  microsoft_graph_app_id             = "00000003-0000-0000-c000-000000000000"
  microsoft_graph_user_read_scope_id = "e1fe6dd8-ba31-4d61-89e7-88639da4683d"

  # OpenID Connect scope IDs for Microsoft Graph
  microsoft_graph_scopes = {
    openid  = "37f7f235-527c-4136-accd-4a02d197296e"
    profile = "14dad69e-099b-42c9-810b-d002981feec1"
    email   = "64a6cdd6-aab1-4aaf-94b8-3cc8405e90d0"
  }

  # Well-known Microsoft client application IDs for developer tools
  microsoft_developer_tools = {
    azure_cli        = "04b07795-8ddb-461a-bbee-02f9e1bf7b46"
    azure_powershell = "1950a258-227b-4e31-a9cf-717495945fc2"
    visual_studio    = "872cd9fa-d31f-45e0-9eab-6e460a02d1f1"
  }

  # Derived paths (binary name matches Cargo package name)
  lambda_binary_path             = "../target/lambda/aws-lambda-mcp/bootstrap"
  interceptor_lambda_binary_path = "../target/lambda/interceptor/bootstrap"
  tool_schema_path               = "../tool_schema.json"

  # Entra ID OAuth configuration
  entra_tenant_id     = data.azuread_client_config.current.tenant_id
  entra_discovery_url = "https://login.microsoftonline.com/${local.entra_tenant_id}/v2.0/.well-known/openid-configuration"

  # Combined redirect URIs: standard defaults + any additional from var.entra_redirect_uris
  combined_redirect_uris = distinct(concat([
    "http://localhost:6274/callback/",
    "https://vscode.dev/redirect",
    "http://127.0.0.1:33418/"
  ], var.entra_redirect_uris))

  # Generate display name and descriptions from project name
  project_display_name = title(replace(local.project_name_with_suffix, "-", " "))

  # ===================================================================
  # Entra App Metadata - Smart Defaults & Auto-Detection
  # ===================================================================

  # Publisher identity: assumes service principal context for CI/CD
  # For local development with user accounts, override entra_created_by_user variable
  publisher_account_name = var.entra_created_by_user != "" ? var.entra_created_by_user : "CI/CD Pipeline (${data.azuread_client_config.current.object_id})"

  # Auto-detect MCP tools from tool_schema.json (full names)
  tool_schema_data    = try(jsondecode(file(local.tool_schema_path)), [])
  auto_detected_tools = [for tool in local.tool_schema_data : tool.name]

  # Computed values with fallbacks
  entra_resolved_app_name   = var.entra_app_name != "" ? var.entra_app_name : var.project_name
  entra_resolved_created_by = var.entra_created_by_user != "" ? var.entra_created_by_user : local.publisher_account_name

  # MCP tools: use override if provided, otherwise auto-detect from schema
  entra_mcp_tools_list   = length(var.entra_mcp_tools_override) > 0 ? var.entra_mcp_tools_override : local.auto_detected_tools
  entra_mcp_tools_joined = length(local.entra_mcp_tools_list) > 0 ? join(",", local.entra_mcp_tools_list) : "none"

  # Date defaults: goLiveDate=today, retireBy=today+30days
  entra_auto_go_live_date   = var.entra_go_live_date != "" ? var.entra_go_live_date : formatdate("YYYY-MM-DD", timestamp())
  entra_auto_retire_by_date = var.entra_retire_by_date != "" ? var.entra_retire_by_date : formatdate("YYYY-MM-DD", timeadd(timestamp(), "720h"))

  # Secrets expiry: auto-compute 2 years from now
  entra_auto_secrets_expiry = var.entra_secrets_expiry_date != "" ? var.entra_secrets_expiry_date : formatdate("YYYY-MM-DD", timeadd(timestamp(), "17520h"))

  # Base Entra app tags (technical characteristics)
  entra_app_base_tags = [
    "agentcore-gateway",
    "oauth2",
    "pkce",
    "terraform-managed"
  ]

  # Metadata tags (governance, lifecycle, discovery)
  entra_app_metadata_tags = [
    "Publisher:${local.publisher_account_name}",
    "Owner:${var.entra_app_owner}",
    "businessUnit:${var.entra_business_unit}",
    "appName:${local.entra_resolved_app_name}",
    "env:${var.entra_environment}",
    "dataClass:${var.entra_data_classification}",
    "piiProcessing:${var.entra_pii_processing}",
    "goLiveDate:${local.entra_auto_go_live_date}",
    "retireBy:${local.entra_auto_retire_by_date}",
    "graphScopes:${var.entra_graph_scopes_summary}",
    "version:${var.entra_app_version}",
    "mcpTools:${local.entra_mcp_tools_joined}",
    "secretsExpiry:${local.entra_auto_secrets_expiry}",
    "createdIn:${var.entra_created_in_tool}",
    "createdBy:${local.entra_resolved_created_by}",
    "installedByCount:${var.entra_installed_by_count}",
  ]

  # Merge base tags with metadata
  entra_app_tags = concat(local.entra_app_base_tags, local.entra_app_metadata_tags)

  # Human-readable notes (semicolon-delimited)
  entra_app_notes = join("; ", [
    "Publisher=${local.publisher_account_name}",
    "Owner=${var.entra_app_owner}",
    "businessUnit=${var.entra_business_unit}",
    "appName=${local.entra_resolved_app_name}",
    "env=${var.entra_environment}",
    "dataClass=${var.entra_data_classification}",
    "piiProcessing=${var.entra_pii_processing}",
    "goLiveDate=${local.entra_auto_go_live_date}",
    "retireBy=${local.entra_auto_retire_by_date}",
    "graphScopes=${var.entra_graph_scopes_summary}",
    "version=${var.entra_app_version}",
    "mcpTools=${local.entra_mcp_tools_joined}",
    "secretsExpiry=${local.entra_auto_secrets_expiry}",
    "createdIn=${var.entra_created_in_tool}",
    "createdBy=${local.entra_resolved_created_by}",
    "installedByCount=${var.entra_installed_by_count}"
  ])

  # Common environment variables for Lambda functions
  common_lambda_env_vars = {
    RUST_LOG = var.rust_log_level
  }
}
