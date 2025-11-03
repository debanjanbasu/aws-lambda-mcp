# OAuth Configuration Summary

## Overview

The infrastructure now uses **Entra ID OAuth 2.0 with PKCE** (Proof Key for Code Exchange) for secure, secretless authentication. All configuration is fully managed by Terraform.

## What Changed

### 1. **Terraform-Managed Identifier URI** ✅
- **Before**: Manually set via Azure CLI after Terraform apply
- **After**: Fully managed by `azuread_application_identifier_uri` resource
- **Benefit**: One-command deployment, no manual steps

### 2. **Removed Hardcoding** ✅
- Extracted constants to `locals.tf`:
  - Microsoft Graph app ID and scope IDs
  - Entra discovery URL construction
  - Gateway audience configuration
- Added variables in `variables.tf` for all customizable values
- **Benefit**: Easy to customize and maintain

### 3. **Proper OAuth Scope Configuration** ✅
- Exposes API scope `access_as_user` on the application
- Sets identifier URI to `api://{client_id}`
- Gateway accepts both `api://{client_id}` and `{client_id}` for compatibility
- **Benefit**: Correct JWT audience validation

## Architecture

```
User → Browser → Entra ID (PKCE flow)
                     ↓ (authorization code)
                 Token exchange
                     ↓ (access token with aud: client_id)
                 Bedrock Gateway (JWT validation)
                     ↓
                 Lambda Function
```

## File Structure

```
iac/
├── providers.tf       # Terraform and provider versions (AWS, Azure AD, Archive, Random)
├── variables.tf       # All configurable variables with defaults
├── locals.tf          # Centralized constants and computed values
├── main.tf           # Lambda, Gateway, IAM resources
├── entra_oauth.tf    # Entra ID app registration + identifier URI
├── outputs.tf        # All outputs for scripts and testing
├── get-token.sh      # OAuth flow automation (updated with correct scope)
├── refresh-token.sh  # Token refresh automation
└── login.sh          # AWS + Azure login helper
```

## Key Resources

### Entra ID Application
- **Resource**: `azuread_application.bedrock_gateway`
- **Features**:
  - Public client with PKCE support
  - OAuth scope: `access_as_user`
  - Microsoft Graph User.Read access
  - Redirect URI: `http://localhost:6274/callback/`

### Application Identifier URI
- **Resource**: `azuread_application_identifier_uri.bedrock_gateway`
- **Value**: `api://{client_id}`
- **Purpose**: Makes tokens have correct audience claim

### Bedrock Gateway
- **Resource**: `aws_bedrockagentcore_gateway.main`
- **Authorizer**: Custom JWT (Entra ID OIDC)
- **Allowed Audiences**: 
  - `api://{client_id}` (standard format)
  - `{client_id}` (compatibility fallback)

## Configuration Variables

All variables have sensible defaults. Override in `terraform.tfvars` if needed:

```hcl
# Entra ID
entra_app_name                = "aws-lambda-mcp"
entra_sign_in_audience        = "AzureADMyOrg"
entra_redirect_uris           = ["http://localhost:6274/callback/"]
entra_oauth_scope_value       = "access_as_user"

# AWS
aws_region           = "ap-southeast-2"
lambda_function_name = "aws-lambda-mcp"
lambda_memory_size   = 128
lambda_timeout       = 30
log_retention_days   = 3
```

## Local Values (Non-Configurable Constants)

Defined in `locals.tf`:

```hcl
locals {
  # Microsoft Graph (well-known GUIDs - never change)
  microsoft_graph_app_id           = "00000003-0000-0000-c000-000000000000"
  microsoft_graph_user_read_scope_id = "e1fe6dd8-ba31-4d61-89e7-88639da4683d"
  
  # Computed from tenant
  entra_tenant_id     = data.azuread_client_config.current.tenant_id
  entra_discovery_url = "https://login.microsoftonline.com/${local.entra_tenant_id}/v2.0/.well-known/openid-configuration"
  
  # Application identifiers
  app_identifier_uri        = "api://${azuread_application.bedrock_gateway.client_id}"
  gateway_allowed_audiences = [local.app_identifier_uri, azuread_application.bedrock_gateway.client_id]
}
```

## Deployment

### One-Command Deploy
```bash
cd .. && make deploy
```

This will:
1. Build ARM64 Lambda with UPX compression
2. Run `terraform init` (if needed)
3. Run `terraform apply`
4. Create Entra app with OAuth scope
5. Set identifier URI automatically
6. Configure Bedrock Gateway with JWT validation

### Get OAuth Token
```bash
cd iac
./get-token.sh
```

This will:
1. Generate PKCE parameters
2. Open browser for authentication
3. Exchange authorization code for tokens
4. Save tokens to `.env`
5. Test the Gateway
6. Optionally launch MCP Inspector

## Token Claims

Access tokens now include:
```json
{
  "aud": "7c5bb0d0-ff35-4d0d-af09-48c8579942e2",  // Client ID (matches Gateway config)
  "iss": "https://login.microsoftonline.com/{tenant}/v2.0",
  "scp": "access_as_user",                        // Our custom scope
  "appid": "7c5bb0d0-ff35-4d0d-af09-48c8579942e2",
  "oid": "{user-object-id}",                      // User identity
  "upn": "user@domain.com"
}
```

## Testing

### List Available Tools
```bash
source .env
curl -X POST "$(terraform output -raw bedrock_gateway_url)" \
  -H "Authorization: Bearer $MCP_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' | jq .
```

### Call a Tool
```bash
curl -X POST "$(terraform output -raw bedrock_gateway_url)" \
  -H "Authorization: Bearer $MCP_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"aws-lambda-mcp-gateway-target___get_weather","arguments":{"location":"Sydney"}},"id":2}' | jq .
```

### Check Token Claims
```bash
source .env
echo "$MCP_ACCESS_TOKEN" | cut -d. -f2 | base64 -d | jq .
```

## Security Features

✅ **Secretless**: PKCE eliminates need for client secrets  
✅ **Short-lived tokens**: 60-minute expiry  
✅ **Refresh tokens**: Long-lived, can be revoked  
✅ **User context**: Every request tied to real user identity  
✅ **MFA support**: Enforced by tenant policies  
✅ **Audit trail**: All auth events in Entra ID logs  
✅ **JWT validation**: OIDC discovery with signature verification

## Troubleshooting

### Token has wrong audience
- **Symptom**: `{"error": {"code": -32001, "message": "Invalid Bearer token"}}`
- **Cause**: Requesting wrong scope
- **Fix**: Use scope `api://{client_id}/access_as_user` in OAuth request

### Identifier URI not set
- **Symptom**: Tokens missing `api://` prefix in audience
- **Fix**: Terraform now handles this automatically via `azuread_application_identifier_uri`

### Terraform circular dependency
- **Symptom**: "Self-referential block" error
- **Fix**: Already resolved - we use separate `azuread_application_identifier_uri` resource

## Outputs

All outputs available via `terraform output`:

```bash
terraform output entra_app_client_id      # OAuth client ID
terraform output entra_tenant_id          # Tenant ID
terraform output entra_discovery_url      # OIDC discovery endpoint
terraform output bedrock_gateway_url      # Gateway API endpoint
terraform output lambda_function_arn      # Lambda ARN
```

## Cost

**Free Tier Coverage**:
- Entra ID OAuth: Free (included with Microsoft 365)
- AWS Lambda: 1M requests/month free
- CloudWatch Logs: 5GB/month free (3-day retention)
- Bedrock Gateway: Pay per request only

**Typical monthly cost**: $0 (within free tier)

## Next Steps

1. **Test authentication**: `./get-token.sh`
2. **Deploy Lambda code**: Fix the internal tool error
3. **Add more tools**: Extend `src/tools/` and regenerate schema
4. **Enable MFA**: Configure in Entra ID tenant settings
5. **Monitor usage**: CloudWatch Logs + Entra ID sign-in logs

---

**Last Updated**: 2025-11-03  
**Terraform Version**: >= 1.0  
**Provider Versions**: AWS >= 5.0, AzureAD >= 3.0, Random >= 3.0
