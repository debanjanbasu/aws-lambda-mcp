# AWS Lambda MCP - Bedrock Gateway Infrastructure

Terraform infrastructure for deploying a Rust Lambda function as a Bedrock Agent Gateway with Entra ID OAuth authentication.

## Quick Start

**One command to login to both AWS and Azure:**

```bash
# Interactive setup script
cd iac
./SETUP.sh
```

This script will:
1. Check prerequisites (AWS CLI, Azure CLI, Terraform)
2. Login to AWS (SSO or credentials)
3. Login to Azure (interactive or service principal)
4. Create Azure service principal for Terraform
5. Export ARM_* environment variables
6. Show you the next steps

**Or manually:**

```bash
# 1. Login to AWS
aws sso login  # Or: aws configure

# 2. Login to Azure and create service principal
az login
SP=$(az ad sp create-for-rbac \
  --name terraform-aws-lambda-mcp \
  --role Contributor \
  --scopes /subscriptions/$(az account show --query id -o tsv) \
  --sdk-auth)

# 3. Export Azure credentials
export ARM_TENANT_ID=$(echo $SP | jq -r .tenantId)
export ARM_CLIENT_ID=$(echo $SP | jq -r .clientId)
export ARM_CLIENT_SECRET=$(echo $SP | jq -r .clientSecret)
export ARM_SUBSCRIPTION_ID=$(echo $SP | jq -r .subscriptionId)

# 4. Build Lambda
cd ..
make release

# 5. Deploy
cd iac
cp terraform.tfvars.example terraform.tfvars
terraform init
terraform apply
```

## What Gets Deployed

### AWS Resources
- **Lambda Function**: ARM64 Rust binary (~1.3MB with UPX compression)
- **Bedrock Agent Gateway**: OAuth-protected endpoint
- **Gateway Target**: Links gateway to Lambda with tool schema
- **CloudWatch Logs**: JSON-formatted logs (3-day retention)
- **IAM Roles**: Minimal permissions

### Azure Resources
- **App Registration**: Multi-tenant OAuth app
- **Service Principal**: For authentication
- **Public Client Support**: PKCE, device code, SPA flows

**Cost**: ~$0/month (within free tier for low usage)

## Authentication Flows

All flows are **secret-less** and secure:

### 1. PKCE (CLI Tools)
```bash
# Browser-based authentication
open "https://login.microsoftonline.com/$TENANT/oauth2/v2.0/authorize?..."
# Exchange code + verifier for token
```

### 2. Device Code (SSH/Terminal)
```bash
# Display code to user
echo "Visit https://microsoft.com/devicelogin"
echo "Enter code: ABCD-1234"
# Poll for token
```

### 3. SPA (Browser Apps)
```javascript
// JavaScript PKCE flow
oauth.login();  // Native browser auth
```

See [OAUTH_FLOWS.md](OAUTH_FLOWS.md) for complete examples.

## Configuration

### Minimal Setup

```hcl
# terraform.tfvars
aws_region           = "ap-southeast-2"
lambda_function_name = "aws-lambda-mcp"
entra_app_name      = "aws-lambda-mcp"
```

### Common Overrides

```hcl
# Increase memory for complex operations
lambda_memory_size = 256

# Extend timeout for slow external APIs
lambda_timeout = 60

# Add redirect URIs for web apps
entra_redirect_uris = [
  "https://app.example.com/callback"
]

# Add SPA URIs for frontend apps
entra_spa_redirect_uris = [
  "https://app.example.com"
]
```

## Architecture

```
User → Entra ID (OAuth) → Bedrock Gateway → Lambda (Rust)
                             ↓
                       OpenID Connect
                         Discovery
```

**Security**:
- Gateway validates JWT tokens via OIDC
- No client secrets (secret-less flows only)
- Multi-tenant support (any Microsoft org)
- User-based authentication (no service accounts)

## Outputs

```bash
terraform output entra_app_client_id      # OAuth client ID
terraform output entra_tenant_id          # Your tenant ID  
terraform output entra_issuer_url         # OIDC issuer
terraform output entra_discovery_url      # OIDC discovery endpoint
terraform output lambda_function_arn      # Lambda ARN
terraform output bedrock_gateway_arn      # Gateway ARN
```

## Cost Optimization

**Free Tier Resources** (no cost):
- Lambda: 1M requests/month + 400,000 GB-seconds
- CloudWatch Logs: 5GB ingestion + 5GB storage
- Bedrock Gateway: No additional charge
- Entra ID OAuth: Free for public client flows

**Cost Drivers** (after free tier):
- Lambda invocations: $0.20 per 1M requests
- Lambda compute: $0.0000133334 per GB-second (ARM64)
- CloudWatch Logs: $0.50 per GB ingested

**Optimization Tips**:
- Use 128MB memory (minimum cost)
- Set log retention to 3 days
- Use ARM64 (Graviton2 - 20% cheaper)
- Compress binary with UPX (faster cold start)
- Don't enable CloudWatch alarms (not in free tier)

**Estimated Monthly Cost**: $0-5 for typical usage

## Monitoring (Free)

Use built-in AWS Console:

```bash
# Lambda console
aws lambda get-function --function-name aws-lambda-mcp

# View logs
aws logs tail /aws/lambda/aws-lambda-mcp --follow

# Check errors
aws cloudwatch get-metric-statistics \
  --namespace AWS/Lambda \
  --metric-name Errors \
  --dimensions Name=FunctionName,Value=aws-lambda-mcp \
  --start-time $(date -u -d '1 hour ago' +%Y-%m-%dT%H:%M:%S) \
  --end-time $(date -u +%Y-%m-%dT%H:%M:%S) \
  --period 3600 --statistics Sum
```

## Updating

### Provider Updates

Versions use `>=` constraints for automatic updates:

```bash
terraform init -upgrade  # Get latest providers
terraform plan           # Check for breaking changes
terraform apply          # Apply if no issues
```

Agentic CI/CD handles breaking changes automatically.

### Lambda Binary

```bash
cd ..
make release
cd iac
terraform apply  # Redeploys with new binary
```

### Tool Schema

```bash
cd ..
make schema      # Regenerate tool_schema.json
cd iac
terraform apply  # Updates gateway with new schema
```

## Troubleshooting

### Error: "No such file or directory: bootstrap"
```bash
cd .. && make release
```

### Error: "Application not found" (Azure)
```bash
# Set Azure credentials
export ARM_TENANT_ID="..."
export ARM_CLIENT_ID="..."
export ARM_CLIENT_SECRET="..."
terraform init
```

### Error: "Schema validation failed"
```bash
cd .. && make schema
jq . tool_schema.json  # Verify valid JSON
```

### Lambda timeout
```hcl
# Increase timeout in terraform.tfvars
lambda_timeout = 60
```

### Out of memory
```hcl
# Increase memory in terraform.tfvars
lambda_memory_size = 256
```

## Files

```
iac/
├── providers.tf         # Terraform and provider config
├── variables.tf         # Input variables
├── main.tf             # Core infrastructure
├── entra_oauth.tf      # Entra ID OAuth resources
├── outputs.tf          # Output values
├── README.md           # This file
├── OAUTH_FLOWS.md      # OAuth implementation examples
└── terraform.tfvars.example  # Configuration template
```

## Security

✅ **No secrets in code**: All flows are cryptographic (PKCE)  
✅ **Multi-tenant**: Works with any Microsoft organization  
✅ **User identity**: Every token represents a real user  
✅ **MFA support**: Enforced by Entra ID policies  
✅ **Audit trail**: All authentications logged  
✅ **Least privilege**: Minimal IAM permissions  
✅ **Encrypted**: All data encrypted at rest and in transit  

## Support

- **Issues**: Create an issue in the repository
- **OAuth Examples**: See [OAUTH_FLOWS.md](OAUTH_FLOWS.md)
- **Cost Details**: See AWS Cost Explorer (tag: `Project=aws-lambda-mcp`)

## License

Same as parent project.

---

**Last Updated**: 2025-11-02  
**Maintained By**: Platform Team + Agentic CI/CD
