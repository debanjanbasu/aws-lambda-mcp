.PHONY: help login init plan apply deploy test-token test-lambda logs clean destroy oauth-config add-redirect-url

help: ## Show this help
	@echo "AWS Lambda MCP - Infrastructure Commands"
	@echo ""
	@echo "Quick Start:"
	@awk 'BEGIN {FS = ":.*?## "}; /^deploy:.*##/ {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "Step by Step:"
	@awk 'BEGIN {FS = ":.*?## "}; /^(init|plan|apply):.*##/ {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "Testing:"
	@awk 'BEGIN {FS = ":.*?## "}; /^(test-token|test-lambda|logs):.*##/ {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "Utilities:"
	@awk 'BEGIN {FS = ":.*?## "}; /^(login|clean|destroy|oauth-config|add-redirect-url|remove-redirect-url):.*##/ {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

login: ## Authenticate AWS + Azure CLIs
	@echo "🔐 Authenticating with AWS and Azure..."
	@command -v aws >/dev/null 2>&1 || (echo "❌ AWS CLI not found. Install: https://aws.amazon.com/cli/" && exit 1)
	@command -v az >/dev/null 2>&1 || (echo "❌ Azure CLI not found. Install: https://aka.ms/azure-cli" && exit 1)
	
	@if aws sts get-caller-identity >/dev/null 2>&1; then \
		echo "✅ AWS: $$(aws sts get-caller-identity --query 'Account' --output text)"; \
	else \
		echo "Please login to AWS:"; \
		if aws configure list | grep -q sso_start_url; then \
			aws sso login; \
		else \
			echo "Run: aws configure"; \
			exit 1; \
		fi; \
	fi
	
	@if az account show >/dev/null 2>&1; then \
		echo "✅ Azure: $$(az account show --query 'name' -o tsv)"; \
	else \
		echo "Logging into Azure..."; \
		az login; \
	fi
	@echo "✅ Authentication complete!"

init: ## Initialize Terraform
	@echo "⚙️  Initializing Terraform..."
	@terraform init -backend-config=backend.config

plan: init ## Show deployment changes
	@echo "📋 Planning infrastructure changes..."
	@terraform plan

apply: init ## Apply changes
	@echo "🚀 Applying infrastructure changes..."
	@terraform apply -auto-approve

deploy: init apply ## Deploy infrastructure (init + apply)
	@echo "✅ Infrastructure deployed successfully!"

test-token: ## Get OAuth token + launch MCP Inspector
	@echo "🔑 Generating OAuth token..."
	@bash -c ' \
	set -e; \
	command -v jq >/dev/null 2>&1 || { echo -e "❌ jq not found. Install: brew install jq" >&2; exit 1; }; \
	command -v curl >/dev/null 2>&1 || { echo -e "❌ curl not found. Install: brew install curl" >&2; exit 1; }; \
	command -v npx >/dev/null 2>&1 || { echo -e "❌ npx not found. Install Node.js and npm." >&2; exit 1; }; \
	CLIENT_ID=$$(terraform output -raw entra_app_client_id 2>/dev/null); \
	TENANT_ID=$$(terraform output -raw entra_tenant_id 2>/dev/null); \
	CLIENT_SECRET=$$(terraform output -raw entra_client_secret 2>/dev/null); \
	if [ -z "$$CLIENT_ID" ] || [ -z "$$TENANT_ID" ] || [ -z "$$CLIENT_SECRET" ]; then \
		echo -e "❌ Missing Terraform outputs. Run \"make apply\" first to deploy infrastructure." >&2; \
		exit 1; \
	fi; \
	SCOPE="api://$$CLIENT_ID/.default"; \
	TOKEN_RESPONSE=$$(curl -s -X POST "https://login.microsoftonline.com/$$TENANT_ID/oauth2/v2.0/token" \
		-H "Content-Type: application/x-www-form-urlencoded" \
		-d "client_id=$$CLIENT_ID" \
		-d "client_secret=$$CLIENT_SECRET" \
		-d "grant_type=client_credentials" \
		-d "scope=$$SCOPE"); \
	if echo "$$TOKEN_RESPONSE" | jq -e ".error" >/dev/null 2>&1; then \
		echo -e "❌ Error getting token:" >&2; \
		echo "$$TOKEN_RESPONSE" | jq -r ".error_description // .error" >&2; \
		exit 1; \
	fi; \
	ACCESS_TOKEN=$$(echo "$$TOKEN_RESPONSE" | jq -r ".access_token"); \
	if [ -z "$$ACCESS_TOKEN" ] || [ "$$ACCESS_TOKEN" = \"null\" ]; then \
		echo -e "❌ Failed to get token. ACCESS_TOKEN is empty or null." >&2; \
		exit 1; \
	fi; \
	echo "# Generated: $$(date)" > .env; \
	echo "export MCP_ACCESS_TOKEN=\"$$ACCESS_TOKEN\"" >> .env; \
	echo "export MCP_CLIENT_ID=\"$$CLIENT_ID\"" >> .env; \
	echo "export MCP_TENANT_ID=\"$$TENANT_ID\"" >> .env; \
	echo -e "✅ Token saved to .env"; \
	GATEWAY_URL=$$(terraform output -raw agentcore_gateway_url 2>/dev/null || echo ""); \
	if [ -n "$$GATEWAY_URL" ] && [ "$$GATEWAY_URL" != \"null\" ]; then \
		if command -v pbcopy >/dev/null 2>&1; then \
			echo "$$ACCESS_TOKEN" | pbcopy; \
			echo -e "📋 Token copied to clipboard"; \
		fi; \
		echo -e "🔗 Gateway URL: $$GATEWAY_URL"; \
		echo -e "🚀 Launching MCP Inspector..."; \
		source .env && npx @modelcontextprotocol/inspector --transport http --server-url "$$GATEWAY_URL" || { echo -e "❌ Failed to launch MCP Inspector." >&2; exit 1; }; \
	else \
		echo -e "⚠️  Gateway not yet deployed. Deploy first with 'make deploy'" >&2; \
		exit 1; \
	fi' 

test-lambda: ## Test Lambda directly
	@echo "\033[1;34m🧪 Testing Lambda function directly...\033[0m"
	@aws lambda invoke \
		--function-name aws-agentcore-gateway \
		--cli-binary-format raw-in-base64-out \
		--payload '{"name":"get_weather","location":"Sydney"}' \
		/tmp/test-response.json > /dev/null
	@echo "Response:"
	@cat /tmp/test-response.json | jq .
	@echo "✅ Lambda test complete!"

logs: ## Tail Lambda logs
	@echo "📜 Tailing Lambda logs (Ctrl+C to exit)..."
	@aws logs tail /aws/lambda/aws-lambda-mcp --follow

clean: ## Remove tokens and backups
	@echo "🧹 Cleaning up temporary files..."
	@rm -f .env
	@rm -f terraform.tfstate.*.backup
	@echo "✅ Clean complete!"

destroy: ## Destroy infrastructure
	@echo "🧨 WARNING: This will destroy ALL infrastructure!"
	@read -p "Type 'destroy' to confirm: " CONFIRM; \
	if [ "$$CONFIRM" != "destroy" ]; then \
		echo "❌ Aborted"; \
		exit 1; \
	fi
	@terraform destroy -auto-approve

oauth-config: ## Display OAuth configuration for any OAuth 2.0 compliant client
	@echo "\033[1;34m🔑 OAuth Configuration Details:\033[0m"
	@bash -c ' \
	CLIENT_ID=$$(terraform output -raw entra_app_client_id 2>/dev/null); \
	TENANT_ID=$$(terraform output -raw entra_tenant_id 2>/dev/null); \
	CLIENT_SECRET=$$(terraform output -raw entra_client_secret 2>/dev/null); \
	if [ -z "$$CLIENT_ID" ] || [ -z "$$TENANT_ID" ]; then \
		echo "\033[1;31m❌ Run \"make apply\" first to deploy infrastructure\033[0m" >&2; \
		exit 1; \
	fi; \
	echo -e "\033[1;32mOAuth 2.0 Configuration:\033[0m"; \
	echo "  Client ID: $$CLIENT_ID"; \
	echo -e "  \033[1;31m🚨 Client Secret: $$CLIENT_SECRET (WARNING: Treat this as a sensitive credential)\033[0m"; \
	echo "  Authority: https://login.microsoftonline.com/$$TENANT_ID"; \
	echo "  Authorization URL: https://login.microsoftonline.com/$$TENANT_ID/oauth2/v2.0/authorize"; \
	echo "  Token URL: https://login.microsoftonline.com/$$TENANT_ID/oauth2/v2.0/token"; \
	echo "  Refresh URL: https://login.microsoftonline.com/$$TENANT_ID/oauth2/v2.0/token"; \
	echo "  Scopes: api://$$CLIENT_ID/.default, User.Read, openid, profile, email"; \
	echo ""; \
	echo -e "\033[1;33mFor service principal authentication:\033[0m"; \
	echo "  1. Use the Client ID and Client Secret for authentication"; \
	echo "  2. Include the scopes in your token request"; \
	echo ""; \
	echo -e "\033[1;33mNote: If configuring with a non-Dynamic Client Registration compatible OAuth provider, you may need to run '\''make add-redirect-url'\'' to manually add redirect URIs.\033[0m";' 

add-redirect-url: ## Add custom OAuth redirect URL to terraform.tfvars
	@echo "🔗 Adding redirect URL to terraform.tfvars..."
	@bash -c ' \
	if [ ! -f terraform.tfvars ]; then \
		echo "❌ terraform.tfvars file not found"; \
		exit 1; \
	fi; \
	read -p "Enter redirect URI to add: " REDIRECT_URI; \
	if [ -z "$$REDIRECT_URI" ]; then \
		echo "❌ Redirect URI cannot be empty"; \
		exit 1; \
	fi; \
	if grep -q """$$REDIRECT_URI""" terraform.tfvars; then \
		echo "⚠️ Redirect URI already exists in terraform.tfvars"; \
		exit 0; \
	fi; \
	awk -v uri="""$$REDIRECT_URI""" '"'"' \
		/entra_redirect_uris = \[/ { \
			in_uris = 1; \
			print; \
			next; \
		} \
		in_uris && /\]/ { \
			print "  " uri ","; \
			in_uris = 0; \
		} \
		{ print } \
	'"'"' terraform.tfvars > terraform.tfvars.tmp && mv terraform.tfvars.tmp terraform.tfvars; \
	echo "✅ Redirect URI added to terraform.tfvars successfully!"; \
	echo "💡 Run 'make deploy' to apply the changes to Entra ID"; \
	' 

remove-redirect-url: ## Remove custom OAuth redirect URL from terraform.tfvars
	@echo "🔗 Removing redirect URL from terraform.tfvars..."
	@bash -c ' \
	if [ ! -f terraform.tfvars ]; then \
		echo "❌ terraform.tfvars file not found"; \
		exit 1; \
	fi; \
	read -p "Enter redirect URI to remove: " REDIRECT_URI; \
	if [ -z "$$REDIRECT_URI" ]; then \
		echo "❌ Redirect URI cannot be empty"; \
		exit 1; \
	fi; \
	if ! grep -q """$$REDIRECT_URI""" terraform.tfvars; then \
		echo "⚠️ Redirect URI not found in terraform.tfvars"; \
		exit 0; \
	fi; \
	awk -v uri="""$$REDIRECT_URI""" '"'"' \
		BEGIN { found = 0 } \
		/entra_redirect_uris = \[/ { in_uris = 1 } \
		in_uris && index($$0, uri) { \
			found = 1; \
			next; \
		} \
		in_uris && /\]/ { in_uris = 0 } \
		{ print } \
		END { if (found == 0) { exit 1 } } \
	'"'"' terraform.tfvars > terraform.tfvars.tmp && mv terraform.tfvars.tmp terraform.tfvars; \
	echo "✅ Redirect URI removed from terraform.tfvars successfully!"; \
	echo "💡 Run 'make deploy' to apply the changes to Entra ID"; \
	'