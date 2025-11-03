# Quick Reference - OAuth & Gateway

## 🚀 Deploy Everything
```bash
cd .. && make deploy
```

## 🔐 Get OAuth Token
```bash
cd iac && ./get-token.sh
```

## 🧪 Test Gateway
```bash
source .env
GATEWAY_URL=$(terraform output -raw bedrock_gateway_url)

# List tools
curl -X POST "$GATEWAY_URL" \
  -H "Authorization: Bearer $MCP_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' | jq .
```

## 📊 Check Logs
```bash
aws logs tail /aws/lambda/aws-lambda-mcp --follow
```

## 🔄 Refresh Token
```bash
./refresh-token.sh
```

## 🗑️ Clean Up
```bash
cd .. && make tf-destroy
```

## 📝 Key Terraform Commands
```bash
terraform plan              # Preview changes
terraform apply             # Apply changes
terraform output            # Show all outputs
terraform output -raw <name>  # Get specific output
terraform destroy           # Remove all resources
```

## 🔍 Debug Token
```bash
source .env
echo "$MCP_ACCESS_TOKEN" | cut -d. -f2 | base64 -d | jq .
```

## 📚 More Info
- Full details: `OAUTH_CONFIGURATION.md`
- Infrastructure: `../INFRASTRUCTURE.md`
- Deployment: `../DEPLOYMENT.md`
