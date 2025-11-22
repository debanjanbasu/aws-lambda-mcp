# Local values - centralized configuration

locals {
  # Project naming with optional suffix
  project_name_with_suffix = var.project_name_suffix == null ? "${var.project_name}-${random_string.project_suffix.result}" : (var.project_name_suffix != "" ? "${var.project_name}-${var.project_name_suffix}" : var.project_name)

  # Microsoft Graph API identifiers (well-known GUIDs)
  microsoft_graph_app_id             = "00000003-0000-0000-c000-000000000000"
  microsoft_graph_user_read_scope_id = "e1fe6dd8-ba31-4d61-89e7-88639da4683d"

  # Microsoft Graph App Role IDs (for application permissions)
  microsoft_graph_user_read_all_app_role_id = "df021288-bdef-4463-88db-98f22de89214"

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
  lambda_binary_path = "../target/lambda/aws-lambda-mcp/bootstrap"
  tool_schema_path   = "../tool_schema.json"

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

  # Generate Entra app tags from project type
  entra_app_tags = ["agentcore-gateway", "oauth2", "pkce", "terraform-managed"]
}
