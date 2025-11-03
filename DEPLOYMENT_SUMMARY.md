# Deployment Summary

✅ **Infrastructure deployed successfully!**

## What's Deployed

### AWS Resources
- **Lambda Function**: `aws-lambda-mcp` (ARM64, UPX-compressed, ~1.3MB)
- **Bedrock Gateway**: MCP protocol endpoint with JWT validation
- **CloudWatch Logs**: 3-day retention for cost optimization
- **IAM Roles**: Minimal permissions (Lambda execution + Gateway invoke)

### Azure Resources
- **Entra ID App**: `aws-lambda-mcp` with PKCE flow (no client secrets)
- **OAuth Scopes**: `/.default`, `offline_access`, `openid`, `profile`
- **Redirect URIs**: `http://localhost:8080/callback/`, `http://localhost:3000/callback/`

## Quick Commands

```bash
# Test the gateway (launches MCP Inspector)
cd iac && ./get-token.sh

# Refresh expired token
cd iac && ./refresh-token.sh

# Redeploy after code changes
make deploy

# View Lambda logs
aws logs tail /aws/lambda/aws-lambda-mcp --follow

# Destroy infrastructure
make tf-destroy
```

## Gateway Details

**Endpoint**: `https://aws-lambda-mcp-gateway-at1k1jz4v4.gateway.bedrock-agentcore.ap-southeast-2.amazonaws.com/mcp`

**Authentication**: Entra ID OAuth 2.0 with PKCE
- No client secrets required
- Uses authorization code flow with PKCE
- Tokens valid for 60 minutes
- Refresh tokens supported

**Authorization**: JWT validation via OIDC discovery
- Issuer: `https://login.microsoftonline.com/1dd8d4ab-c281-4982-992e-c0036c9bff72/v2.0`
- Discovery: `.well-known/openid-configuration`

## OAuth Flow

```
1. User runs ./get-token.sh
2. Browser opens → Microsoft login
3. User authenticates with Entra ID
4. Authorization code returned to localhost
5. Script exchanges code for token using PKCE
6. Access token + refresh token saved to .env
7. MCP Inspector launches automatically
```

## Cost Estimate

**Monthly cost** (typical dev usage): **$0.00**
- Lambda: Free tier covers 1M requests
- CloudWatch: Free tier covers 5GB logs
- Bedrock Gateway: Pay-per-request (no standing charges)
- Entra ID: Free for OAuth flows

## Security Features

✅ Secretless authentication (PKCE)
✅ JWT validation with OIDC discovery
✅ User-level authentication (no shared credentials)
✅ MFA supported via Entra ID policies
✅ Audit trail in Entra ID sign-in logs
✅ Minimal IAM permissions
✅ Short-lived tokens (60 min)

## Terraform Outputs

Access via: `cd iac && terraform output <name>`

- `entra_app_client_id`: OAuth client ID
- `entra_tenant_id`: Azure tenant ID
- `bedrock_gateway_url`: Gateway endpoint
- `lambda_function_arn`: Lambda ARN
- `entra_discovery_url`: OIDC discovery URL

## Next Steps

1. **Test the gateway**: `cd iac && ./get-token.sh`
2. **Add more tools**: Edit `src/tools/*.rs` and run `make deploy`
3. **Monitor usage**: Check Lambda metrics in AWS Console
4. **Review costs**: CloudWatch dashboard for invocation counts

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Invalid token | Run `./refresh-token.sh` |
| Schema mismatch | Run `make deploy` to rebuild |
| Lambda timeout | Increase `lambda_timeout` in `iac/terraform.tfvars` |
| OAuth consent error | Accept permissions in browser during `./get-token.sh` |

## Files

- `iac/main.tf`: Lambda + Gateway + IAM infrastructure
- `iac/entra_oauth.tf`: Entra ID app registration
- `iac/get-token.sh`: OAuth flow + Inspector launcher
- `iac/refresh-token.sh`: Token refresh utility
- `iac/login.sh`: AWS + Azure authentication helper
- `iac/README.md`: Complete documentation

---

**Deployment Date**: $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Region**: ap-southeast-2 (Sydney)
**Managed By**: Terraform + Agentic CI/CD
