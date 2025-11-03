# Infrastructure Summary

Complete Terraform infrastructure for deploying AWS Lambda as Bedrock Agent Core Gateway with Entra ID OAuth (PKCE).

## Architecture

```
MCP Client → Entra ID OAuth → Bedrock Gateway (MCP) → Lambda (Rust) → External APIs
                 (PKCE)              ↓
                            JWT Validation (OIDC)
```

**Components**:
- AWS Lambda (ARM64, Rust, ~1.3MB with UPX)
- Bedrock Agent Core Gateway (MCP protocol)
- Entra ID OAuth (PKCE, no secrets)
- CloudWatch Logs (3-day retention, cost-optimized)
- IAM roles (least privilege)

## Quick Start

```bash
# From root directory
./iac/login.sh         # Login to AWS + Azure
make deploy            # Build + Deploy
./iac/get-token.sh     # Get token + launch Inspector
```

## Files Created

### Terraform (`iac/`)
- **main.tf** - Lambda, Gateway, IAM roles, CloudWatch
- **entra_oauth.tf** - Entra ID app (SPA with PKCE)
- **providers.tf** - AWS/Azure providers (latest versions)
- **variables.tf** - Configuration with sensible defaults
- **outputs.tf** - Gateway URL, client IDs, tenant info

### Scripts
- **login.sh** - AWS + Azure authentication helper
- **get-token.sh** - OAuth PKCE flow + launch MCP Inspector
- **refresh-token.sh** - Refresh token + relaunch Inspector

### Documentation
- **iac/README.md** - Complete guide with troubleshooting

### Makefile Integration
- `make deploy` - Build + regenerate schema + apply Terraform
- `make tf-init` - Initialize Terraform
- `make tf-plan` - Plan changes (builds Lambda first)
- `make tf-apply` - Apply changes (builds Lambda first)
- `make tf-destroy` - Destroy all resources

## Security

✅ **Secretless**: PKCE OAuth (no client secrets)  
✅ **JWT validation**: Every request validated via OIDC  
✅ **User identity**: Tokens represent real users  
✅ **MFA support**: Enforced by Entra ID policy  
✅ **Audit trail**: All auth events logged  
✅ **Least privilege**: Minimal IAM permissions  

## Cost

**Free Tier** (typical = $0/month):
- Lambda: 1M requests + 400K GB-seconds
- CloudWatch: 5GB ingestion + storage
- Gateway: Pay-per-request
- OAuth: Free (public client)

**Optimizations**:
- ARM64 (20% cheaper)
- UPX compression (65% smaller)
- 3-day log retention
- No CloudWatch alarms ($0.10 each)

## Configuration

All variables have defaults. Override in `iac/terraform.tfvars`:

```hcl
aws_region          = "ap-southeast-2"  # Default
project_name        = "aws-lambda-mcp"  # Default
lambda_memory_size  = 128               # Default
lambda_timeout      = 30                # Default
log_retention_days  = 3                 # Default
```

## Testing

### Automated
```bash
./iac/get-token.sh  # Opens browser → Authenticates → Launches Inspector
```

### Manual
```bash
source iac/.env
curl -X POST "$(cd iac && terraform output -raw bedrock_gateway_url)" \
  -H "Authorization: Bearer $MCP_ACCESS_TOKEN" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

## Development Workflow

```bash
# 1. Edit code
vim src/tools/my_tool.rs

# 2. Deploy (builds + schema + terraform)
make deploy

# 3. Test
./iac/refresh-token.sh  # Relaunches Inspector
```

## Troubleshooting

| Error | Fix |
|-------|-----|
| Bootstrap not found | `make release` |
| Invalid token | `./iac/refresh-token.sh` |
| Schema error | `make deploy` |
| Lambda timeout | Add `lambda_timeout = 60` to `terraform.tfvars` |

### Check Logs
```bash
aws logs tail /aws/lambda/aws-lambda-mcp --follow
```

## Production Checklist

- [ ] Review `iac/variables.tf` defaults
- [ ] Set custom redirect URIs in `terraform.tfvars`
- [ ] Enable CloudTrail for audit logs
- [ ] Configure AWS Config for compliance
- [ ] Review Entra ID sign-in logs
- [ ] Test token refresh flow
- [ ] Document team access procedures

## Documentation

- **iac/README.md** - Complete architecture, config, troubleshooting
- **AGENTS.md** - Code style guide for developers
- **INFRASTRUCTURE.md** - This file (overview)

---

**Status**: ✅ Production Ready  
**License**: MIT/Apache-2.0
