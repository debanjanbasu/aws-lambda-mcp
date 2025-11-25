# Entra ID OAuth Configuration for Amazon Bedrock AgentCore Gateway
# Uses Public Client Application for authorization code flow with PKCE (no secrets)

# Data source to get current Entra ID configuration
data "azuread_client_config" "current" {}

# Generate a random suffix for unique resource names if not provided
resource "random_string" "project_suffix" {
  length  = 6
  lower   = true
  upper   = false
  numeric = true
  special = false
}

# Generate a stable UUID for the OAuth scope
resource "random_uuid" "oauth_scope" {}

# Create Entra ID Application Registration for AgentCore Gateway
resource "azuread_application" "agentcore_app" {
  display_name     = local.project_name_with_suffix
  owners           = [data.azuread_client_config.current.object_id]
  sign_in_audience = var.entra_sign_in_audience

  # Expose API with default scope for our app to receive tokens with correct audience
  api {
    requested_access_token_version = 2

    oauth2_permission_scope {
      admin_consent_description  = "Allow the application to access ${local.project_display_name} on behalf of the signed-in user"
      admin_consent_display_name = "Access ${local.project_display_name}"
      enabled                    = true
      id                         = random_uuid.oauth_scope.result
      type                       = "User"
      user_consent_description   = "Allow the application to access ${local.project_display_name} on behalf of the signed-in user"
      user_consent_display_name  = "Access ${local.project_display_name}"
      value                      = var.entra_oauth_scope_value
    }

    # Pre-authorize common Microsoft developer tools to access this API without requiring admin consent
    # This enables seamless local development and testing workflows
    known_client_applications = values(local.microsoft_developer_tools)
  }

  # Public client configuration - supports authorization code with PKCE
  public_client {
    redirect_uris = local.combined_redirect_uris
  }

  # Web application configuration - supports client credentials with secret
  web {
    redirect_uris = local.combined_redirect_uris

    implicit_grant {
      access_token_issuance_enabled = false
      id_token_issuance_enabled     = false
    }
  }

  # Required resource access - Microsoft Graph for user info and OpenID scopes
  # IMPORTANT: Azure AD scope combination rules:
  # When requesting tokens, the '.default' scope for a custom API (e.g., api://{client_id}/.default)
  # CANNOT be combined with other explicit scopes (e.g., User.Read, openid, profile, email)
  # in the same request.
  #
  # Choose one approach for client applications:
  # 1. Request only the custom API's '.default' scope: "api://{client_id}/.default"
  #    (This grants all permissions defined for the custom API)
  # 2. Request explicit scopes, including custom API scopes (e.g., "api://{client_id}/YourCustomScope")
  #    alongside standard scopes like "User.Read openid profile email".
  #    DO NOT include '.default' in this case.
  required_resource_access {
    resource_app_id = local.microsoft_graph_app_id

    # User.Read.All - basic profile access (app role version)
    resource_access {
      id   = local.microsoft_graph_user_read_all_app_role_id
      type = "Role"
    }

    # OpenID Connect scopes (openid, profile, email)
    dynamic "resource_access" {
      for_each = local.microsoft_graph_scopes
      content {
        id   = resource_access.value
        type = "Scope"
      }
    }
  }

  group_membership_claims = var.entra_group_membership_claims

  # Optional claims for access tokens - include user email and name
  optional_claims {
    access_token {
      name = "email"
    }
    access_token {
      name = "family_name"
    }
    access_token {
      name = "given_name"
    }
  }

  # Client secret for OAuth 2.0 confidential clients
  # Note: Using password block within azuread_application instead of separate
  # azuread_application_password resource, as the latter was deprecated/removed
  # in azuread provider 3.7.0
  password {
    display_name = "OAuth 2.0 Confidential Client"
    end_date     = timeadd(timestamp(), "17520h") # 2 years
  }

  tags = local.entra_app_tags

  # Ignore changes to redirect URIs so they can be managed externally
  # This allows adding redirect URIs via Azure Portal or other tools
  # without Terraform trying to remove them on subsequent applies
  # Also ignore changes to owners to prevent conflicts with manual owner management
  # Owners may be modified by CI/CD pipelines, Azure administrators, or other automation
  # that manages application access and permissions outside of Terraform
  lifecycle {
    ignore_changes = [
      web[0].redirect_uris,
      public_client[0].redirect_uris,
      owners,
      password,
    ]
  }
}
