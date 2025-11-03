# Local values - centralized configuration

locals {
  # Microsoft Graph API identifiers (well-known GUIDs)
  microsoft_graph_app_id           = "00000003-0000-0000-c000-000000000000"
  microsoft_graph_user_read_scope_id = "e1fe6dd8-ba31-4d61-89e7-88639da4683d"
  
  # Entra ID OAuth configuration
  entra_tenant_id = data.azuread_client_config.current.tenant_id
  entra_discovery_url = "https://login.microsoftonline.com/${local.entra_tenant_id}/v2.0/.well-known/openid-configuration"
  
  # Application identifier URI - api://{client_id}
  app_identifier_uri = "api://${azuread_application.bedrock_gateway.client_id}"
  
  # Gateway allowed audiences - accept both formats for compatibility
  gateway_allowed_audiences = [
    local.app_identifier_uri,
    azuread_application.bedrock_gateway.client_id
  ]
}
