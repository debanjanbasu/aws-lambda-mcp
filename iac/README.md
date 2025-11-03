# AWS Lambda MCP - Bedrock Gateway Infrastructure

Terraform infrastructure for deploying a Rust Lambda as a Bedrock Agent Core Gateway with Entra ID OAuth (PKCE flow - no secrets required).

## Quick Start

```bash
# 1. Login to both clouds
./login.sh

# 2. Build and deploy (from project root)
cd .. && make deploy

# 3. Get OAuth token and test with MCP Inspector
cd iac && ./get-token.sh
```

The gateway includes: Lambda (ARM64, UPX-compressed), Bedrock Gateway (MCP protocol), Entra ID OAuth (PKCE), CloudWatch logs, JWT validation, and minimal IAM permissions.

## Architecture

```
MCP Client → Entra ID (PKCE) → Bedrock Gateway (JWT) → Lambda → APIs
```

All authentication is secretless using PKCE flow with JWT validation via OIDC discovery.

## Prerequisites

```bash
# Install required tools
brew install awscli azure-cli terraform jq
cargo install cargo-lambda
brew install upx  # Binary compression

# Authenticate
./login.sh  # Logs into both AWS and Azure
```

## Configuration

All variables have defaults. Override in `terraform.tfvars` if needed:

```hcl
aws_region          = "ap-southeast-2"  # Default region
lambda_function_name = "aws-lambda-mcp" # Follows project name
lambda_memory_size  = 128               # Minimal for cost
lambda_timeout      = 30
log_retention_days  = 3                 # Minimal for cost
```

## Deployment Commands

```bash
# From project root
make deploy          # Build release + deploy infrastructure

# Or step by step
make release        # Build ARM64 Lambda with UPX
cd iac && terraform init
cd iac && terraform apply
```

## Testing

### Get OAuth Token & Launch Inspector
```bash
cd iac
./get-token.sh      # Opens browser → authenticate → launches Inspector
```

Inspector opens at http://localhost:6543 with token pre-configured.

### Refresh Token
```bash
./refresh-token.sh  # Refreshes expired token + relaunches Inspector
```

### Manual API Calls
```bash
source .env
GATEWAY_URL=$(terraform output -raw bedrock_gateway_url)

# List tools
curl -X POST "$GATEWAY_URL" \
  -H "Authorization: Bearer $MCP_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' | jq .

# Call tool
curl -X POST "$GATEWAY_URL" \
  -H "Authorization: Bearer $MCP_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_weather","arguments":{"location":"Sydney"}},"id":1}' | jq .
```

### Debugging
```bash
# Watch Lambda logs
aws logs tail /aws/lambda/aws-lambda-mcp --follow

# Check token claims
source .env && echo "$MCP_ACCESS_TOKEN" | cut -d. -f2 | base64 -d | jq .

# Test without auth (expect 401)
curl -X POST "$GATEWAY_URL" -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

## Cost Optimization

**Free Tier** (typical usage = $0/month):
- Lambda: 1M requests + 400,000 GB-seconds/month
- CloudWatch: 5GB logs (3-day retention)
- Bedrock Gateway: Pay-per-request only
- Entra ID: Free for OAuth

**Optimizations Applied**:
- ARM64/Graviton (20% cheaper)
- UPX compression (65% smaller binary)
- 128MB memory (minimal)
- 3-day log retention
- No CloudWatch alarms ($0.10 each, use Logs Insights instead)

## Terraform Outputs

```bash
terraform output entra_app_client_id   # OAuth client ID
terraform output entra_tenant_id       # Tenant ID
terraform output bedrock_gateway_url   # Gateway endpoint
terraform output lambda_function_arn   # Lambda ARN
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Bootstrap not found | `cd .. && make release` |
| Invalid token | `./refresh-token.sh` |
| Schema mismatch | `cd .. && make deploy` |
| Lambda timeout | Set `lambda_timeout = 60` in tfvars |
| OAuth consent required | Accept permissions in `./get-token.sh` |

## File Structure

```
iac/
├── providers.tf       # AWS + Azure providers (auto-update)
├── variables.tf       # Variables with defaults
├── main.tf           # Lambda + Gateway + IAM
├── entra_oauth.tf    # Entra ID app (PKCE)
├── outputs.tf        # All outputs
├── login.sh          # Cloud login helper
├── get-token.sh      # OAuth + Inspector launcher
├── refresh-token.sh  # Token refresh + Inspector
└── README.md         # This file
```

## Security Features

- ✅ **Secretless**: PKCE eliminates client secrets
- ✅ **User identity**: Every request tied to real user
- ✅ **JWT validation**: OIDC discovery + validation
- ✅ **MFA support**: Enforced by tenant policy
- ✅ **Audit trail**: All auth in Entra ID logs
- ✅ **Short-lived tokens**: 60-minute expiry
- ✅ **Least privilege IAM**: Minimal permissions

## Cleanup

```bash
cd .. && make tf-destroy  # Remove all infrastructure
cd .. && cargo clean      # Remove build artifacts
```

## Updating

```bash
# Code changes
cd .. && make deploy

# Provider upgrades (auto-accepts latest)
terraform init -upgrade && terraform apply
```

---

**Maintained by agentic CI/CD** • Built with Terraform, Rust, Lambda, Bedrock, Entra ID
