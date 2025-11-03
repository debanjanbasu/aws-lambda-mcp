# AWS Lambda MCP Gateway - Deployment Guide

## ✅ Infrastructure Ready

The Terraform infrastructure is configured and validated for deploying the AWS Lambda MCP Gateway with:
- **AWS Bedrock Agent Core Gateway** (MCP protocol)
- **Entra ID OAuth** (PKCE flow, no secrets)
- **Lambda Function** (Rust, ARM64, UPX-compressed)
- **CloudWatch Logging** (3-day retention)

## Quick Deploy

```bash
# 1. Build Lambda binary
make release

# 2. Login to clouds
cd iac && ./login.sh

# 3. Deploy infrastructure
cd .. && make deploy

# 4. Test with OAuth token
cd iac && ./get-token.sh
```

## What's Deployed

### AWS Resources
- **Lambda Function** (`aws-lambda-mcp`)
  - Runtime: `provided.al2023`
  - Architecture: ARM64
  - Memory: 128MB
  - Timeout: 30s
  - Handler: `bootstrap`

- **Bedrock Agent Core Gateway** (`aws-lambda-mcp-gateway`)
  - Protocol: MCP
  - Authorizer: CUSTOM_JWT (Entra ID)
  - Discovery URL: `https://login.microsoftonline.com/{tenant}/v2.0/.well-known/openid-configuration`
  
- **Gateway Target**
  - Lambda invocation
  - Tool schemas from `tool_schema.json`
  - IAM role-based credentials

- **IAM Roles**
  - Lambda execution role (basic execution policy)
  - Gateway role (Lambda invoke permission)

- **CloudWatch Log Group**
  - Path: `/aws/lambda/aws-lambda-mcp`
  - Retention: 3 days

### Azure Resources
- **Entra ID App Registration** (`aws-lambda-mcp`)
  - Public client (PKCE flow)
  - Redirect URI: `http://localhost:6274/callback/`
  - Required permissions: User.Read (Microsoft Graph)
  - Token version: v2

## Configuration Files

```
iac/
├── providers.tf       # Terraform & provider versions (>= 5.0)
├── variables.tf       # Input variables with defaults
├── main.tf           # All infrastructure resources
├── entra_oauth.tf    # Entra ID app registration
├── outputs.tf        # Output values for testing
├── login.sh          # Cloud authentication helper
├── get-token.sh      # OAuth token + MCP Inspector launcher
└── refresh-token.sh  # Token refresh helper
```

## Authentication Flow

### 1. User Authentication (PKCE)
```bash
./get-token.sh
```
- Opens browser to Entra ID
- User authenticates with MFA
- Authorization code returned to localhost
- Script exchanges code for tokens using PKCE verifier
- Tokens saved to `.env`

### 2. API Request
```
Client → Bedrock Gateway (JWT validation) → Lambda → Response
```
- Gateway validates JWT via OIDC discovery
- Checks audience matches client_id
- Invokes Lambda with validated user context

## Cost Analysis

### Expected Costs (FREE TIER)
- **Lambda**: $0/month (within 1M requests, 400K GB-seconds)
- **Bedrock Gateway**: $0/month (pay-per-request only)
- **CloudWatch Logs**: $0/month (within 5GB ingestion/storage)
- **Entra ID**: $0/month (free for OAuth)

### Cost Optimizations Applied
- ARM64 architecture (20% cheaper than x86)
- UPX binary compression (faster cold starts)
- Minimal memory (128MB)
- Short log retention (3 days)
- No CloudWatch alarms ($0.10 each)
- No NAT Gateway/VPC
- No API Gateway

## Testing

### Get OAuth Token
```bash
cd iac
./get-token.sh
```
**Output**: Browser opens → authenticate → MCP Inspector launches

### Test Gateway Manually
```bash
source .env
GATEWAY_URL=$(terraform output -raw bedrock_gateway_url)

# List tools
curl -X POST "$GATEWAY_URL" \
  -H "Authorization: Bearer $MCP_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' | jq .
```

### Refresh Expired Token
```bash
./refresh-token.sh
```

## Outputs

After deployment, get configuration:
```bash
cd iac

# All outputs
terraform output

# Specific values
terraform output entra_app_client_id
terraform output bedrock_gateway_url
terraform output lambda_function_arn
```

Key outputs:
- `entra_app_client_id`: OAuth client ID
- `entra_tenant_id`: Azure tenant ID
- `bedrock_gateway_url`: Gateway endpoint for MCP Inspector
- `bedrock_gateway_id`: Gateway ID
- `lambda_function_arn`: Lambda ARN

## Troubleshooting

### Error: "Missing tool_schema.json"
```bash
make schema
```

### Error: "Bootstrap not found"
```bash
make release
```

### Error: "Invalid Bearer token"
```bash
cd iac && ./get-token.sh
```

### Error: "Entra app not found"
Ensure Azure AD permissions. May require admin consent:
```bash
az ad app permission admin-consent --id <CLIENT_ID>
```

### View Logs
```bash
aws logs tail /aws/lambda/aws-lambda-mcp --follow
```

## Security

✅ **Implemented**:
- PKCE flow (no client secrets)
- JWT validation via OIDC discovery
- User-level authentication (MFA supported)
- IAM least privilege roles
- No public endpoints
- Short-lived tokens (60 min)
- Audit trail in Entra ID logs

## Updating

### Code Changes
```bash
make deploy
```
Terraform detects binary hash changes and updates Lambda.

### Schema Changes
```bash
make schema
make deploy
```
Terraform detects schema changes and updates gateway target.

### Provider Upgrades
```bash
cd iac
terraform init -upgrade
terraform apply
```

## Cleanup

```bash
# Destroy all infrastructure
make tf-destroy

# Or manually
cd iac && terraform destroy
```

**Note**: Entra ID app may persist. Delete manually if needed:
```bash
az ad app delete --id $(terraform output -raw entra_app_client_id)
```

## Next Steps

1. **Deploy**: Run `make deploy` from project root
2. **Test**: Run `cd iac && ./get-token.sh`
3. **Monitor**: Check CloudWatch logs
4. **Extend**: Add more tools in `src/tools/`

## CI/CD Integration

```yaml
# Example GitHub Actions workflow
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: aarch64-unknown-linux-gnu
      
      - name: Install cargo-lambda
        run: pip install cargo-lambda
      
      - name: Login to AWS
        uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: ${{ secrets.AWS_ROLE_ARN }}
          aws-region: ap-southeast-2
      
      - name: Login to Azure
        run: |
          az login --service-principal \
            -u ${{ secrets.ARM_CLIENT_ID }} \
            -p ${{ secrets.ARM_CLIENT_SECRET }} \
            --tenant ${{ secrets.ARM_TENANT_ID }}
      
      - name: Deploy
        run: make deploy
```

## Support

- **Documentation**: `/iac/README.md`
- **Logs**: CloudWatch `/aws/lambda/aws-lambda-mcp`
- **Issues**: Check Terraform output and logs

---

**Status**: ✅ Ready for deployment  
**Last Updated**: 2025-11-03  
**Terraform Version**: >= 1.0  
**AWS Provider**: >= 5.0  
**Azure Provider**: >= 3.0
