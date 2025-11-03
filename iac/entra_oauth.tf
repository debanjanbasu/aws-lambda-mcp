# Entra ID (Azure AD) OAuth Configuration for Bedrock Gateway
# The gateway uses OpenID Connect discovery URL for authentication

# Data source to get current Azure AD configuration
data "azuread_client_config" "current" {}

# Create Entra ID Application Registration
resource "azuread_application" "bedrock_gateway" {
  display_name     = var.entra_app_name
  owners           = [data.azuread_client_config.current.object_id]
  sign_in_audience = "AzureADMultipleOrgs" # Allow any Microsoft Entra ID tenant

  # Application ID URI (audience claim)
  identifier_uris = ["api://${var.entra_app_name}"]

  # Use v2 access tokens (allows more flexible identifier URIs)
  api {
    requested_access_token_version = 2
  }

  # Enable public client flows (device code, PKCE)
  public_client {
    redirect_uris = var.entra_public_client_redirect_uris
  }

  # Web platform configuration for OAuth (authorization code + PKCE)
  web {
    redirect_uris = var.entra_redirect_uris

    implicit_grant {
      access_token_issuance_enabled = false
      id_token_issuance_enabled     = false
    }
  }

  # Single-page application platform (PKCE only, no client secret)
  single_page_application {
    redirect_uris = var.entra_spa_redirect_uris
  }

  # Optional: Group membership claims
  group_membership_claims = ["SecurityGroup"]

  tags = [
    "bedrock-gateway",
    "oauth2",
    "terraform-managed"
  ]
}

# Update application with identifier URI after creation
resource "azuread_application_identifier_uri" "bedrock_gateway" {
  application_id = azuread_application.bedrock_gateway.id
  identifier_uri = "api://${azuread_application.bedrock_gateway.client_id}"
}

# Create Service Principal for the application
resource "azuread_service_principal" "bedrock_gateway" {
  client_id                    = azuread_application.bedrock_gateway.client_id
  app_role_assignment_required = false
  owners                       = [data.azuread_client_config.current.object_id]

  tags = [
    "bedrock-gateway",
    "oauth2",
    "terraform-managed"
  ]
}
