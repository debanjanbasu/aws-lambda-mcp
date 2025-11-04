# AWS Lambda MCP - Infrastructure

Terraform infrastructure for Rust Lambda + Bedrock Gateway + Entra ID OAuth (PKCE).

## Quick Start

```bash
make login       # Authenticate AWS + Azure
make deploy      # Deploy infrastructure
make test-token  # Get OAuth token + launch MCP Inspector
```

## Architecture

```
MCP Client → Entra ID (PKCE) → Bedrock Gateway (JWT) → Lambda → APIs
```

Secretless PKCE flow, JWT validation via OIDC, ARM64 Lambda with UPX compression.

## Configuration

All variables have defaults in `variables.tf`. Override in `terraform.tfvars`:

```hcl
entra_sign_in_audience = "AzureADMultipleOrgs"  # Any Entra ID tenant
lambda_memory_size     = 128                     # Minimal for cost
lambda_timeout         = 30
log_retention_days     = 3
rust_log_level         = "info"                  # info (prod), debug/trace (troubleshooting)
gateway_enable_debug   = false                   # Enable for detailed errors
```

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

### Gateway Debug Logging

Enable detailed error messages for troubleshooting:

```hcl
# In terraform.tfvars
gateway_enable_debug = true
```

Then redeploy:
```bash
terraform apply -auto-approve
```

When enabled, Gateway returns detailed error messages with full context. When disabled (default), only minimal error information is provided.

**Note**: Current AWS provider only supports DEBUG level or standard error messages. Set back to `false` for production.

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
```

## Manual Testing

```bash
source .env
curl -X POST "$(terraform output -raw bedrock_gateway_url)" \
  -H "Authorization: Bearer $MCP_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

## Files

- `main.tf` - Lambda, Gateway, IAM
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

**Cost**: Free tier covers typical usage ($0/month)
