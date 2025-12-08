# AWS Lambda MCP - Infrastructure

Terraform infrastructure for Rust Lambda + Bedrock Gateway + Entra ID OAuth (PKCE).

## Quick Start

```bash
make login       # Authenticate AWS + Azure
make deploy      # Deploy infrastructure
make test-token  # Get OAuth token + launch MCP Inspector
```

## GitHub Actions Setup

This repository uses GitHub Actions for CI/CD. When used as a template repository, the following secrets must be configured:

- `AWS_IAM_ROLE_ARN` - AWS IAM Role for GitHub Actions OIDC authentication
- `AZURE_CLIENT_ID` - Entra ID App Registration Client ID
- `AZURE_TENANT_ID` - Entra ID Tenant ID
- `TF_BACKEND_BUCKET` - S3 Bucket for Terraform state storage (uses native S3 locking)

Configure these secrets in your repository's Settings > Secrets and variables > Actions.

## Architecture

```
MCP Client → Entra ID (PKCE) → Bedrock Gateway (JWT) → Interceptor Lambda → Main Lambda → APIs
                                      ↓                              ↓
                              JWT Validation (OIDC)        Header Propagation & Token Exchange
```

Secretless PKCE flow, JWT validation via OIDC, ARM64 Lambdas with UPX compression, native Terraform gateway interceptor configuration.

## Configuration

All variables have defaults in `variables.tf`. Override in `terraform.tfvars`:

```hcl
entra_sign_in_audience  = "AzureADMultipleOrgs"  # Any Entra ID tenant
lambda_memory_size      = 128                     # Minimum memory (128MB)
lambda_timeout          = 30                      # Standard timeout (30s)
lambda_concurrent_executions      = 100           # Main Lambda: 100, Interceptor: 200 (2x)
log_retention_days      = 3                       # Short retention to minimize costs
rust_log_level          = "info"                  # info (prod), debug/trace (troubleshooting)
gateway_exception_level = null                    # Gateway exception level (DEBUG/INFO/WARN/ERROR/null)
```

### Redirect URI Management

The infrastructure automatically includes standard development redirect URIs:
- `http://localhost:6274/callback/` (MCP Inspector)
- `https://vscode.dev/redirect` (VS Code authentication)
- `http://127.0.0.1:33418/` (Standard OAuth development)

Simply add all your redirect URIs to the `entra_redirect_uris` variable:

```hcl
# In terraform.tfvars
entra_redirect_uris = [
  "http://localhost:6274/callback/",
  "https://global.consent.azure-apim.net/redirect/cr324-5fagentcore-2dweather-2dmcp-5fac2271d378fbd65b",
  "https://global.consent.azure-apim.net/redirect/cr324-5faws-20weather-20mcp-20server-5fac2271d378fbd65b",
  # Add any other redirect URIs you need
]
```

This prevents these URIs from being removed during deployments from different environments.

### Lambda Debug Logging

Control Lambda logging verbosity for security and troubleshooting:

```hcl
# Production (secure - no event payloads in logs)
rust_log_level = "info"   # Default - logs only event size

# Troubleshooting (detailed - includes full event payloads)
rust_log_level = "debug"  # or "trace" for maximum detail
```

**Security implications:**
- `info/warn/error`: Only logs event size. Event payloads excluded from logs automatically.
- `debug/trace`: Logs full event payloads and context. Use only for troubleshooting, not production.

### Gateway Exception Logging

Control Gateway exception logging verbosity:

```hcl
# In terraform.tfvars

# Disabled (default) - Minimal error information for security
gateway_exception_level = null

# Error level - Only error messages
gateway_exception_level = "ERROR"

# Warning level - Warning and error messages
gateway_exception_level = "WARN"

# Info level - Informational, warning, and error messages
gateway_exception_level = "INFO"

# Debug level - Most verbose logging (use only for troubleshooting)
gateway_exception_level = "DEBUG"
```

Then redeploy:
```bash
terraform apply -auto-approve
```

**Security considerations:**
Higher verbosity levels may expose sensitive information in error responses. 
Use DEBUG/INFO only for troubleshooting, not in production environments.

Set back to `null` for production deployments.

## Available Commands

```bash
make help         # Show all commands
make login        # Authenticate AWS + Azure CLIs
make init         # Initialize Terraform
make plan         # Show deployment changes
make apply        # Deploy infrastructure
make deploy       # Full deploy (init + apply)
make test-token   # Get OAuth token + launch MCP Inspector
make refresh      # Refresh expired access token
make test-lambda  # Test Lambda directly (bypass Gateway)
make logs         # Tail Lambda logs
make clean        # Remove tokens and backups
make destroy      # Destroy infrastructure
make add-redirect-url  # Add custom redirect URI to terraform.tfvars
```

**Note**: When working with ephemeral environments, you can also use:
- `terraform init -backend-config=../backend-preview.tfvars` for preview environments

## Manual Testing

```bash
source .env
curl -X POST "$(terraform output -raw bedrock_gateway_url)" \
  -H "Authorization: Bearer $MCP_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

## Files

- `main.tf` - Main Lambda, Gateway, IAM, interceptor configuration
- `entra_oauth.tf` - Entra ID app (PKCE)
- `variables.tf` - All configuration
- `outputs.tf` - Client ID, tenant, gateway URL
- `locals.tf` - Constants (Graph IDs, discovery URL)

## Troubleshooting

```bash
# Lambda logs
make logs

# Token claims
source .env && echo "$MCP_ACCESS_TOKEN" | cut -d. -f2 | base64 -d | jq .

# Rebuild Lambda
cd .. && make deploy
```

### State Lock Issues

If you encounter a "state lock" error:

**Automatic Recovery (GitHub Actions)**: The workflow automatically detects and clears stale locks from cancelled runs.

**Manual Recovery (Local Development)**:
```bash
# List current lock (if any)
cd iac && terraform plan

# If locked, force unlock (use the Lock ID from error message)
terraform force-unlock <LOCK_ID>
```

**Common Causes**:
- Cancelled GitHub workflow mid-operation
- Interrupted local Terraform command (Ctrl+C)
- Network timeout during Terraform operation

**Prevention**: The preview environment workflow uses `cancel-in-progress: true`, which may leave stale locks when workflows are cancelled. The reusable Terraform workflow now automatically detects and clears these locks.

**Cost**: Free tier covers typical usage ($0/month)
