.PHONY: help login init plan apply deploy test-token refresh-token test-lambda clean

help:
	@echo "AWS Lambda MCP - Infrastructure Commands"
	@echo ""
	@echo "Setup:"
	@echo "  make login       - Authenticate AWS + Azure CLIs"
	@echo "  make init        - Initialize Terraform"
	@echo "  make plan        - Show deployment changes"
	@echo "  make apply       - Deploy infrastructure"
	@echo "  make deploy      - Full deploy (init + apply)"
	@echo ""
	@echo "Testing:"
	@echo "  make test-token  - Get OAuth token + launch MCP Inspector"
	@echo "  make refresh     - Refresh expired access token"
	@echo "  make test-lambda - Test Lambda directly (bypass Gateway)"
	@echo "  make logs        - Tail Lambda logs"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean       - Remove tokens and state backups"
	@echo "  make destroy     - Destroy infrastructure"

login:
	@echo "=== AWS Lambda MCP - Login Helper ==="
	@echo ""
	@command -v aws >/dev/null 2>&1 || (echo "❌ AWS CLI not found. Install: https://aws.amazon.com/cli/" && exit 1)
	@command -v az >/dev/null 2>&1 || (echo "❌ Azure CLI not found. Install: https://aka.ms/azure-cli" && exit 1)
	@echo "1️⃣  Checking AWS login..."
	@if aws sts get-caller-identity >/dev/null 2>&1; then \
		echo "✅ Already logged in to AWS:"; \
		aws sts get-caller-identity --query 'Account' --output text; \
	else \
		echo "Please login to AWS:"; \
		if aws configure list | grep -q sso_start_url; then \
			aws sso login; \
		else \
			echo "Run: aws configure"; \
			echo "Then try again."; \
			exit 1; \
		fi \
	fi
	@echo ""
	@echo "2️⃣  Checking Azure login..."
	@if az account show >/dev/null 2>&1; then \
		echo "✅ Already logged in to Azure:"; \
		az account show --query 'name' -o tsv; \
	else \
		echo "Opening browser for Azure login..."; \
		az login; \
	fi
	@echo ""
	@echo "✅ Login complete!"

init:
	@echo "Initializing Terraform..."
	@terraform init

plan: init
	@terraform plan

apply: init
	@terraform apply -auto-approve

deploy: init apply
	@echo "✅ Deployment complete!"

test-token:
	@bash -c ' \
	set -e; \
	echo "=== AWS Lambda MCP Gateway - OAuth Token Generator ==="; \
	echo ""; \
	CLIENT_ID=$$(terraform output -raw entra_app_client_id 2>/dev/null); \
	TENANT_ID=$$(terraform output -raw entra_tenant_id 2>/dev/null); \
	CLIENT_SECRET=$$(terraform output -raw entra_client_secret 2>/dev/null); \
	if [ -z "$$CLIENT_ID" ] || [ -z "$$TENANT_ID" ] || [ -z "$$CLIENT_SECRET" ]; then \
		echo "❌ Error: Could not read Terraform outputs"; \
		echo "Run \"make apply\" first"; \
		exit 1; \
	fi; \
	echo "Client ID: $$CLIENT_ID"; \
	echo "Tenant ID: $$TENANT_ID"; \
	echo ""; \
	echo "Getting access token (client credentials flow)..."; \
	SCOPE="api://$$CLIENT_ID/.default"; \
	TOKEN_RESPONSE=$$(curl -s -X POST "https://login.microsoftonline.com/$$TENANT_ID/oauth2/v2.0/token" \
		-H "Content-Type: application/x-www-form-urlencoded" \
		-d "client_id=$$CLIENT_ID" \
		-d "client_secret=$$CLIENT_SECRET" \
		-d "grant_type=client_credentials" \
		-d "scope=$$SCOPE"); \
	if echo "$$TOKEN_RESPONSE" | jq -e ".error" >/dev/null 2>&1; then \
		echo "❌ Error:"; \
		echo "$$TOKEN_RESPONSE" | jq -r ".error_description // .error"; \
		exit 1; \
	fi; \
	ACCESS_TOKEN=$$(echo "$$TOKEN_RESPONSE" | jq -r ".access_token"); \
	if [ -z "$$ACCESS_TOKEN" ] || [ "$$ACCESS_TOKEN" = "null" ]; then \
		echo "❌ Failed to get token"; \
		echo "$$TOKEN_RESPONSE"; \
		exit 1; \
	fi; \
	echo "✅ Token acquired!"; \
	echo ""; \
	echo "# Generated: $$(date)" > .env; \
	echo "export MCP_ACCESS_TOKEN=\"$$ACCESS_TOKEN\"" >> .env; \
	echo "export MCP_CLIENT_ID=\"$$CLIENT_ID\"" >> .env; \
	echo "export MCP_TENANT_ID=\"$$TENANT_ID\"" >> .env; \
	echo "Tokens saved to .env"; \
	echo ""; \
	GATEWAY_URL=$$(terraform output -raw agentcore_gateway_url 2>/dev/null || echo ""); \
	if [ -z "$$GATEWAY_URL" ] || [ "$$GATEWAY_URL" = "null" ]; then \
		echo "⚠️  Gateway not yet deployed"; \
		echo ""; \
		echo "Token saved to .env. To test manually after deployment:"; \
		echo "  source .env"; \
		echo "  npx @modelcontextprotocol/inspector $$GATEWAY_URL"; \
		exit 0; \
	fi; \
	echo "=================================================="; \
	echo "🚀 Launching MCP Inspector"; \
	echo "=================================================="; \
	echo ""; \
	echo "Gateway URL: $$GATEWAY_URL"; \
	echo ""; \
	if command -v pbcopy >/dev/null 2>&1; then \
		echo "$$ACCESS_TOKEN" | pbcopy; \
		echo "✅ Token copied to clipboard (macOS)"; \
	elif command -v xclip >/dev/null 2>&1; then \
		echo "$$ACCESS_TOKEN" | xclip -selection clipboard; \
		echo "✅ Token copied to clipboard (Linux)"; \
	elif command -v xsel >/dev/null 2>&1; then \
		echo "$$ACCESS_TOKEN" | xsel --clipboard --input; \
		echo "✅ Token copied to clipboard (Linux)"; \
	elif command -v clip.exe >/dev/null 2>&1; then \
		echo "$$ACCESS_TOKEN" | clip.exe; \
		echo "✅ Token copied to clipboard (WSL)"; \
	else \
		echo "⚠️  No clipboard tool found"; \
	fi; \
	echo ""; \
	echo "📋 Full Access Token:"; \
	echo "$$ACCESS_TOKEN"; \
	echo ""; \
	echo "Instructions:"; \
	echo "   1. Click Connect in the Inspector"; \
	echo "   2. Paste the token in the Bearer Token field"; \
	echo "   3. Click Connect to authenticate"; \
	echo ""; \
	npx @modelcontextprotocol/inspector --transport http --server-url "$$GATEWAY_URL"; \
	'

refresh:
	@echo "Refresh not needed with client credentials flow"
	@echo "Run 'make test-token' to get a new token"

test-lambda:
	@echo "=== Testing Lambda Directly (bypassing Gateway) ==="
	@echo ""
	@echo "Testing Sydney weather"
	@aws lambda invoke \
		--function-name aws-agentcore-gateway \
		--cli-binary-format raw-in-base64-out \
		--payload '{"name":"get_weather","location":"Sydney"}' \
		/tmp/test-response.json > /dev/null
	@echo "Response:"
	@cat /tmp/test-response.json | jq .
	@echo ""
	@echo "✅ Lambda is working correctly!"

logs:
	@echo "Tailing Lambda logs (Ctrl+C to exit)..."
	@aws logs tail /aws/lambda/aws-lambda-mcp --follow

clean:
	@echo "Cleaning tokens and backups..."
	@rm -f .env
	@rm -f terraform.tfstate.*.backup
	@echo "✅ Clean complete"

destroy:
	@echo "⚠️  This will destroy all infrastructure!"
	@terraform destroy
