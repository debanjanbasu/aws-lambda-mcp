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
	@terraform apply

deploy: init apply
	@echo "✅ Deployment complete!"

test-token:
	@bash -c ' \
	set -e; \
	echo "=== AWS Lambda MCP Gateway - OAuth Token Generator ==="; \
	echo ""; \
	CLIENT_ID=$$(terraform output -raw entra_app_client_id 2>/dev/null); \
	TENANT_ID=$$(terraform output -raw entra_tenant_id 2>/dev/null); \
	APP_NAME=$$(terraform output -raw entra_app_name 2>/dev/null); \
	if [ -z "$$CLIENT_ID" ] || [ -z "$$TENANT_ID" ]; then \
		echo "❌ Error: Could not read Terraform outputs"; \
		echo "Run \"make apply\" first"; \
		exit 1; \
	fi; \
	echo "Client ID: $$CLIENT_ID"; \
	echo "Tenant ID: $$TENANT_ID"; \
	echo "App Name: $$APP_NAME"; \
	echo ""; \
	echo "Generating PKCE parameters..."; \
	CODE_VERIFIER=$$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-43); \
	CODE_CHALLENGE=$$(echo -n "$$CODE_VERIFIER" | openssl dgst -sha256 -binary | base64 | tr "+" "-" | tr "/" "_" | tr -d "="); \
	REDIRECT_URI="http://localhost:6274/callback/"; \
	SCOPE="api://$$CLIENT_ID/access_as_user openid profile email offline_access"; \
	AUTH_URL="https://login.microsoftonline.com/$$TENANT_ID/oauth2/v2.0/authorize"; \
	AUTH_URL="$$AUTH_URL?client_id=$$CLIENT_ID"; \
	AUTH_URL="$$AUTH_URL&response_type=code"; \
	AUTH_URL="$$AUTH_URL&redirect_uri=$$(printf %s "$$REDIRECT_URI" | jq -sRr @uri)"; \
	AUTH_URL="$$AUTH_URL&response_mode=query"; \
	AUTH_URL="$$AUTH_URL&scope=$$(printf %s "$$SCOPE" | jq -sRr @uri)"; \
	AUTH_URL="$$AUTH_URL&code_challenge=$$CODE_CHALLENGE"; \
	AUTH_URL="$$AUTH_URL&code_challenge_method=S256"; \
	echo ""; \
	echo "=================================================="; \
	echo "🔐 STEP 1: Authenticate in Browser"; \
	echo "=================================================="; \
	echo ""; \
	if command -v open >/dev/null 2>&1; then \
		open "$$AUTH_URL"; \
	elif command -v xdg-open >/dev/null 2>&1; then \
		xdg-open "$$AUTH_URL"; \
	else \
		echo "Open this URL:"; \
		echo "$$AUTH_URL"; \
	fi; \
	echo "After authentication, you have two options:"; \
	echo ""; \
	echo "Option 1: Save URL to file (recommended for long URLs):"; \
	echo "  echo 'PASTE_URL_HERE' > /tmp/redirect_url.txt"; \
	echo "  Press Enter here when done"; \
	echo ""; \
	echo "Option 2: Paste URL directly (may truncate):"; \
	echo "  Just paste the URL and press Enter"; \
	echo ""; \
	echo "Waiting for redirect URL..."; \
	IFS= read -r REDIRECT_RESPONSE < /dev/tty; \
	if [ -z "$$REDIRECT_RESPONSE" ] && [ -f /tmp/redirect_url.txt ]; then \
		REDIRECT_RESPONSE=$$(cat /tmp/redirect_url.txt); \
		rm -f /tmp/redirect_url.txt; \
		echo "Read URL from file"; \
	fi; \
	if [[ "$$REDIRECT_RESPONSE" =~ code=([^&]+) ]]; then \
		AUTH_CODE="$${BASH_REMATCH[1]}"; \
	else \
		echo "❌ Could not extract authorization code"; \
		exit 1; \
	fi; \
	echo ""; \
	echo "=================================================="; \
	echo "🔐 STEP 2: Exchange Code for Token"; \
	echo "=================================================="; \
	echo ""; \
	TOKEN_RESPONSE=$$(curl -s -X POST "https://login.microsoftonline.com/$$TENANT_ID/oauth2/v2.0/token" \
		-H "Content-Type: application/x-www-form-urlencoded" \
		-d "client_id=$$CLIENT_ID" \
		-d "grant_type=authorization_code" \
		-d "code=$$AUTH_CODE" \
		-d "redirect_uri=$$REDIRECT_URI" \
		-d "code_verifier=$$CODE_VERIFIER"); \
	if echo "$$TOKEN_RESPONSE" | jq -e ".error" >/dev/null 2>&1; then \
		echo "❌ Error:"; \
		echo "$$TOKEN_RESPONSE" | jq -r ".error_description // .error"; \
		exit 1; \
	fi; \
	ACCESS_TOKEN=$$(echo "$$TOKEN_RESPONSE" | jq -r ".access_token"); \
	REFRESH_TOKEN=$$(echo "$$TOKEN_RESPONSE" | jq -r ".refresh_token // empty"); \
	if [ -z "$$ACCESS_TOKEN" ] || [ "$$ACCESS_TOKEN" = "null" ]; then \
		echo "❌ Failed to get token"; \
		echo "$$TOKEN_RESPONSE"; \
		exit 1; \
	fi; \
	echo "✅ Success!"; \
	echo ""; \
	echo "# Generated: $$(date)" > .env; \
	echo "export MCP_ACCESS_TOKEN=\"$$ACCESS_TOKEN\"" >> .env; \
	echo "export MCP_REFRESH_TOKEN=\"$$REFRESH_TOKEN\"" >> .env; \
	echo "export MCP_CLIENT_ID=\"$$CLIENT_ID\"" >> .env; \
	echo "export MCP_TENANT_ID=\"$$TENANT_ID\"" >> .env; \
	echo "Tokens saved to .env"; \
	echo ""; \
	echo "=================================================="; \
	echo "🚀 Launching MCP Inspector"; \
	echo "=================================================="; \
	echo ""; \
	GATEWAY_URL=$$(terraform output -raw agentcore_gateway_url 2>/dev/null || echo ""); \
	if [ -z "$$GATEWAY_URL" ] || [ "$$GATEWAY_URL" = "null" ]; then \
		echo "⚠️  Gateway not yet deployed"; \
		echo ""; \
		echo "Tokens saved to .env. To test manually after deployment:"; \
		echo "  source .env"; \
		echo "  npx @modelcontextprotocol/inspector <gateway-url> --authorization-token \"Bearer \$$MCP_ACCESS_TOKEN\""; \
		exit 0; \
	fi; \
	echo "Gateway URL: $$GATEWAY_URL"; \
	echo ""; \
	npx @modelcontextprotocol/inspector --transport http --server-url "$$GATEWAY_URL" --header "Authorization: Bearer $$ACCESS_TOKEN"; \
	'

refresh:
	@bash -c ' \
	set -e; \
	echo "=== Refreshing OAuth Access Token ==="; \
	echo ""; \
	if [ -f .env ]; then \
		source .env; \
	fi; \
	if [ -z "$$MCP_REFRESH_TOKEN" ] || [ -z "$$MCP_CLIENT_ID" ] || [ -z "$$MCP_TENANT_ID" ]; then \
		echo "❌ Missing credentials. Run \"make test-token\" first."; \
		exit 1; \
	fi; \
	echo "Refreshing access token..."; \
	TOKEN_RESPONSE=$$(curl -s -X POST "https://login.microsoftonline.com/$$MCP_TENANT_ID/oauth2/v2.0/token" \
		-H "Content-Type: application/x-www-form-urlencoded" \
		-d "client_id=$$MCP_CLIENT_ID" \
		-d "grant_type=refresh_token" \
		-d "refresh_token=$$MCP_REFRESH_TOKEN" \
		-d "scope=$$MCP_CLIENT_ID/.default offline_access"); \
	if echo "$$TOKEN_RESPONSE" | jq -e ".error" >/dev/null 2>&1; then \
		echo "❌ Error:"; \
		echo "$$TOKEN_RESPONSE" | jq -r ".error_description // .error"; \
		echo ""; \
		echo "Refresh token expired. Run \"make test-token\""; \
		exit 1; \
	fi; \
	NEW_ACCESS_TOKEN=$$(echo "$$TOKEN_RESPONSE" | jq -r ".access_token"); \
	NEW_REFRESH_TOKEN=$$(echo "$$TOKEN_RESPONSE" | jq -r ".refresh_token // empty"); \
	if [ -z "$$NEW_REFRESH_TOKEN" ] || [ "$$NEW_REFRESH_TOKEN" = "null" ]; then \
		NEW_REFRESH_TOKEN="$$MCP_REFRESH_TOKEN"; \
	fi; \
	if [ -z "$$NEW_ACCESS_TOKEN" ] || [ "$$NEW_ACCESS_TOKEN" = "null" ]; then \
		echo "❌ Failed to refresh"; \
		exit 1; \
	fi; \
	echo "✅ Success!"; \
	echo ""; \
	echo "# Generated: $$(date)" > .env; \
	echo "export MCP_ACCESS_TOKEN=\"$$NEW_ACCESS_TOKEN\"" >> .env; \
	echo "export MCP_REFRESH_TOKEN=\"$$NEW_REFRESH_TOKEN\"" >> .env; \
	echo "export MCP_CLIENT_ID=\"$$MCP_CLIENT_ID\"" >> .env; \
	echo "export MCP_TENANT_ID=\"$$MCP_TENANT_ID\"" >> .env; \
	echo "Token saved to .env"; \
	echo ""; \
	echo "=================================================="; \
	echo "🚀 Launching MCP Inspector"; \
	echo "=================================================="; \
	echo ""; \
	GATEWAY_URL=$$(terraform output -raw agentcore_gateway_url 2>/dev/null || echo ""); \
	if [ -z "$$GATEWAY_URL" ] || [ "$$GATEWAY_URL" = "null" ]; then \
		echo "⚠️  Gateway not deployed"; \
		exit 0; \
	fi; \
	echo "Gateway: $$GATEWAY_URL"; \
	echo ""; \
	npx @modelcontextprotocol/inspector --transport http --server-url "$$GATEWAY_URL" --header "Authorization: Bearer $$NEW_ACCESS_TOKEN"; \
	'

test-lambda:
	@echo "=== Testing Lambda Directly (bypassing Gateway) ==="
	@echo ""
	@echo "Test 1: Sydney weather"
	@aws lambda invoke \
		--function-name aws-agentcore-gateway \
		--cli-binary-format raw-in-base64-out \
		--payload '{"name":"get_weather","location":"Sydney"}' \
		/tmp/test-response.json > /dev/null
	@echo "Response:"
	@cat /tmp/test-response.json | jq .
	@echo ""
	@echo "Test 2: Sydney weather (again)"
	@aws lambda invoke \
		--function-name aws-agentcore-gateway \
		--cli-binary-format raw-in-base64-out \
		--payload '{"name":"get_weather","location":"Sydney"}' \
		/tmp/test-response2.json > /dev/null
	@echo "Response:"
	@cat /tmp/test-response2.json | jq .
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
