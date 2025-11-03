# Entra ID (Azure AD) OAuth Configuration for Bedrock Gateway
# Uses Public Client Application for authorization code flow with PKCE (no secrets)

# Data source to get current Azure AD configuration
data "azuread_client_config" "current" {}

# Generate a stable UUID for the OAuth scope
resource "random_uuid" "oauth_scope" {}

# Create Entra ID Application Registration
resource "azuread_application" "bedrock_gateway" {
  display_name     = var.entra_app_name
  owners           = [data.azuread_client_config.current.object_id]
  sign_in_audience = var.entra_sign_in_audience

  # Expose API with default scope for our app to receive tokens with correct audience
  api {
    requested_access_token_version = 2
    
    oauth2_permission_scope {
      admin_consent_description  = var.entra_oauth_scope_admin_description
      admin_consent_display_name = var.entra_oauth_scope_admin_name
      enabled                    = true
      id                         = random_uuid.oauth_scope.result
      type                       = "User"
      user_consent_description   = var.entra_oauth_scope_user_description
      user_consent_display_name  = var.entra_oauth_scope_user_name
      value                      = var.entra_oauth_scope_value
    }
  }

  # Public client configuration - supports authorization code with PKCE
  public_client {
    redirect_uris = var.entra_redirect_uris
  }

  # Required resource access - Microsoft Graph for user info
  required_resource_access {
    resource_app_id = local.microsoft_graph_app_id

    resource_access {
      id   = local.microsoft_graph_user_read_scope_id
      type = "Scope"
    }
  }

  group_membership_claims = var.entra_group_membership_claims

  tags = var.entra_app_tags
}

# Update application with identifier_uris (requires app to exist first)
# This is the proper Terraform way to handle self-referential dependencies
resource "azuread_application_identifier_uri" "bedrock_gateway" {
  application_id = azuread_application.bedrock_gateway.id
  identifier_uri = "api://${azuread_application.bedrock_gateway.client_id}"
}
