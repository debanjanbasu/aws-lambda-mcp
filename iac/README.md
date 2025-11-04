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
```

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
