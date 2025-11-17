# Entra ID OAuth Configuration for Amazon Bedrock AgentCore Gateway
# Uses Public Client Application for authorization code flow with PKCE (no secrets)

# Data source to get current Entra ID configuration
data "azuread_client_config" "current" {}

# Generate a stable UUID for the OAuth scope
resource "random_uuid" "oauth_scope" {}

# Create Entra ID Application Registration for AgentCore Gateway
resource "azuread_application" "agentcore_app" {
  display_name     = var.project_name
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

    # openid - OpenID Connect authentication
    resource_access {
      id   = local.openid_scope_id
      type = "Scope"
    }

    # profile - Access to user's profile claims (given_name, family_name)
    resource_access {
      id   = local.profile_scope_id
      type = "Scope"
    }

    # email - Access to user's email address
    resource_access {
      id   = local.email_scope_id
      type = "Scope"
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

  tags = local.entra_app_tags
}

# Grant organization-wide admin consent for Microsoft Graph permissions using app role assignments
# This pre-approves the required app roles so users don't see consent prompts
# NOTE: Converting delegated permissions to app roles
# App roles are typically used for application-only permissions, not user-delegated ones
# Only User.Read has an app role equivalent (User.Read.All), others (openid, profile, email) are OIDC scopes only
resource "azuread_app_role_assignment" "graph_permissions" {
  app_role_id         = local.microsoft_graph_user_read_all_app_role_id
  principal_object_id = data.azuread_client_config.current.object_id
  resource_object_id  = data.azuread_service_principal.microsoft_graph.object_id
}

# Client secret for OAuth 2.0 confidential clients
# 2 year expiry - minimum practical duration for Entra ID
resource "azuread_application_password" "oauth_client" {
  application_id = azuread_application.agentcore_app.id
  display_name   = "OAuth 2.0 Confidential Client"
  end_date       = timeadd(timestamp(), "17520h") # 2 years

  lifecycle {
    ignore_changes = [end_date]
  }
}

# Data source for Microsoft Graph service principal
data "azuread_service_principal" "microsoft_graph" {
  client_id = local.microsoft_graph_app_id
}
