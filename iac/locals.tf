# Local values - centralized configuration

locals {
  # Microsoft Graph API identifiers (well-known GUIDs)
  microsoft_graph_app_id             = "00000003-0000-0000-c000-000000000000"
  microsoft_graph_user_read_scope_id = "e1fe6dd8-ba31-4d61-89e7-88639da4683d"

  # OpenID Connect scope IDs for Microsoft Graph
  openid_scope_id  = "37f7f235-527c-4136-accd-4a02d197296e"
  profile_scope_id = "14dad69e-099b-42c9-810b-d002981feec1"
  email_scope_id   = "64a6cdd6-aab1-4aaf-94b8-3cc8405e90d0"

  # Derived paths (binary name matches Cargo package name)
  lambda_binary_path = "../target/lambda/aws-lambda-mcp/bootstrap"
  tool_schema_path   = "../tool_schema.json"

  # Entra ID OAuth configuration
  entra_tenant_id     = data.azuread_client_config.current.tenant_id
  entra_discovery_url = "https://login.microsoftonline.com/${local.entra_tenant_id}/v2.0/.well-known/openid-configuration"

  # Generate display name and descriptions from project name
  project_display_name     = title(replace(var.project_name, "-", " "))
  oauth_scope_display_name = "Access ${local.project_display_name}"
  oauth_scope_description  = "Allow the application to access ${local.project_display_name} on behalf of the signed-in user"

  # Generate Entra app tags from project type
  entra_app_tags = ["agentcore-gateway", "oauth2", "pkce", "terraform-managed"]

  # Application identifier URI - api://{client_id}
  app_identifier_uri = "api://${azuread_application.agentcore_app.client_id}"

  # Gateway allowed audiences - accept both formats for compatibility
  gateway_allowed_audiences = [
    local.app_identifier_uri,
    azuread_application.agentcore_app.client_id
  ]
}
