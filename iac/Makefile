.PHONY: help login init plan apply deploy test-token test-lambda logs clean destroy oauth-config add-redirect-url

# Colors for output
RED := \033[1;31m
GREEN := \033[1;32m
YELLOW := \033[1;33m
BLUE := \033[1;34m
CYAN := \033[1;36m
BOLD := \033[1m
RESET := \033[0m

help: ## Show this help
	@echo "$(CYAN)AWS Lambda MCP - Infrastructure Commands$(RESET)"
	@echo ""
	@echo "$(GREEN)Quick Start:$(RESET)"
	@awk 'BEGIN {FS = ":.*?## "}; /^deploy:.*##/ {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(GREEN)Step by Step:$(RESET)"
	@awk 'BEGIN {FS = ":.*?## "}; /^(init|plan|apply):.*##/ {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(GREEN)Testing:$(RESET)"
	@awk 'BEGIN {FS = ":.*?## "}; /^(test-token|test-lambda|logs):.*##/ {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(GREEN)Utilities:$(RESET)"
	@awk 'BEGIN {FS = ":.*?## "}; /^(login|clean|destroy|oauth-config|add-redirect-url|remove-redirect-url):.*##/ {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}' $(MAKEFILE_LIST)

login: ## Authenticate AWS + Azure CLIs
	@echo "$(BLUE)🔐 Authenticating with AWS and Azure...$(RESET)"
	@command -v aws >/dev/null 2>&1 || (echo "$(RED)❌ AWS CLI not found. Install: https://aws.amazon.com/cli/$(RESET)" && exit 1)
	@command -v az >/dev/null 2>&1 || (echo "$(RED)❌ Azure CLI not found. Install: https://aka.ms/azure-cli$(RESET)" && exit 1)
	
	@if aws sts get-caller-identity >/dev/null 2>&1; then \
		echo "$(GREEN)✅ AWS: $$(aws sts get-caller-identity --query 'Account' --output text)$(RESET)"; \
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
		echo "$(GREEN)✅ Azure: $$(az account show --query 'name' -o tsv)$(RESET)"; \
	else \
		echo "Logging into Azure..."; \
		az login; \
	fi
	@echo "$(GREEN)✅ Authentication complete!$(RESET)"

init: ## Initialize Terraform
	@echo "$(BLUE)⚙️  Initializing Terraform...$(RESET)"; \
	if [ -f terraform.tfvars.temp ]; then \
		echo "$(CYAN)📝 Using temporary terraform.tfvars...$(RESET)"; \
		cp terraform.tfvars.temp terraform.tfvars; \
	elif [ ! -f terraform.tfvars ]; then \
		echo "$(CYAN)📝 Creating default terraform.tfvars...$(RESET)"; \
		echo "rust_log_level      = \"info\"" > terraform.tfvars; \
		echo "entra_redirect_uris = []" >> terraform.tfvars; \
	fi; \
	terraform init -backend-config=backend.config

plan: init ## Show deployment changes
	@echo "$(BLUE)📋 Planning infrastructure changes...$(RESET)"
	@terraform plan
	@echo "$(GREEN)✅ Plan complete! Review the output above.$(RESET)"

apply: init ## Apply changes
	@echo "$(BLUE)🚀 Applying infrastructure changes...$(RESET)"
	@terraform apply -auto-approve
	@echo "$(GREEN)✅ Infrastructure changes applied successfully!$(RESET)"

deploy: init apply ## Deploy infrastructure (init + apply)
	@echo "$(GREEN)✅ Infrastructure deployed successfully!$(RESET)"

test-token: ## Get OAuth token + launch MCP Inspector
	@echo "$(BLUE)🔑 Generating OAuth token...$(RESET)"
	@bash -c ' \
	set -e; \
	command -v curl >/dev/null 2>&1 || { echo -e "$(RED)❌ curl not found. Install: brew install curl$(RESET)" >&2; exit 1; }; \
	command -v npx >/dev/null 2>&1 || { echo -e "$(RED)❌ npx not found. Install Node.js and npm.$(RESET)" >&2; exit 1; }; \
	CLIENT_ID=$$(terraform output -raw entra_app_client_id 2>/dev/null); \
	TENANT_ID=$$(terraform output -raw entra_tenant_id 2>/dev/null); \
	CLIENT_SECRET=$$(terraform output -raw entra_client_secret 2>/dev/null); \
	CLIENT_CREDENTIALS_SCOPE=$$(terraform output -raw entra_app_scope_client_credentials 2>/dev/null); \
	if [ -z "$$CLIENT_ID" ] || [ -z "$$TENANT_ID" ] || [ -z "$$CLIENT_SECRET" ] || [ -z "$$CLIENT_CREDENTIALS_SCOPE" ]; then \
		echo "$(RED)❌ Missing Terraform outputs. Run \"make apply\" first to deploy infrastructure.$(RESET)" >&2; \
		exit 1; \
	fi; \
	SCOPE="$${CLIENT_CREDENTIALS_SCOPE}"; \
	TOKEN_RESPONSE=$$(curl -s -X POST "https://login.microsoftonline.com/$$TENANT_ID/oauth2/v2.0/token" \
		-H "Content-Type: application/x-www-form-urlencoded" \
		-d "client_id=$$CLIENT_ID" \
		-d "client_secret=$$CLIENT_SECRET" \
		-d "grant_type=client_credentials" \
		-d "scope=$$SCOPE"); \
	if echo "$$TOKEN_RESPONSE" | jq -e ".error" >/dev/null 2>&1; then \
		echo -e "$(RED)❌ Error getting token:$(RESET)" >&2; \
		echo "$$TOKEN_RESPONSE" | jq -r ".error_description // .error" >&2; \
		exit 1; \
	fi; \
	ACCESS_TOKEN=$$(echo "$$TOKEN_RESPONSE" | jq -r ".access_token"); \
	if [ -z "$$ACCESS_TOKEN" ] || [ "$$ACCESS_TOKEN" = \"null\" ]; then \
		echo -e "$(RED)❌ Failed to get token. ACCESS_TOKEN is empty or null.$(RESET)" >&2; \
		exit 1; \
	fi; \
	echo "# Generated: $$(date)" > .env; \
	echo "export MCP_ACCESS_TOKEN=\"$$ACCESS_TOKEN\"" >> .env; \
	echo "export MCP_CLIENT_ID=\"$$CLIENT_ID\"" >> .env; \
	echo "export MCP_TENANT_ID=\"$$TENANT_ID\"" >> .env; \
	echo -e "$(GREEN)✅ Token saved to .env$(RESET)"; \
	GATEWAY_URL=$$(terraform output -raw agentcore_gateway_url 2>/dev/null || echo ""); \
	if [ -n "$$GATEWAY_URL" ] && [ "$$GATEWAY_URL" != \"null\" ]; then \
		if command -v pbcopy >/dev/null 2>&1; then \
			echo "$$ACCESS_TOKEN" | pbcopy; \
			echo -e "$(CYAN)📋 Token copied to clipboard$(RESET)"; \
		fi; \
		echo -e "$(CYAN)🔗 Gateway URL: $$GATEWAY_URL$(RESET)"; \
		echo -e "$(BLUE)🚀 Launching MCP Inspector...$(RESET)"; \
		source .env && npx @modelcontextprotocol/inspector --transport http --server-url "$$GATEWAY_URL" || { echo -e '\''$(RED)❌ Failed to launch MCP Inspector.$(RESET)'\'' >&2; exit 1; }; \
	else \
		echo -e "$(YELLOW)⚠️  Gateway not yet deployed. Deploy first with '\''make deploy'\''.$(RESET)" >&2; \
		exit 1; \
	fi'

test-lambda: ## Test Lambda directly
	@echo "$(BLUE)🧪 Testing Lambda function directly...$(RESET)"
	@aws lambda invoke \
		--function-name aws-agentcore-gateway \
		--cli-binary-format raw-in-base64-out \
		--payload '{"name":"get_weather","location":"Sydney"}' \
		/tmp/test-response.json > /dev/null
	@echo "Response:"
	@cat /tmp/test-response.json | jq .
	@echo "$(GREEN)✅ Lambda test complete!$(RESET)"

logs: ## Tail Lambda logs
	@echo "$(BLUE)📜 Tailing Lambda logs (Ctrl+C to exit)...$(RESET)"
	@LOG_GROUP_NAME=$$(terraform output -raw cloudwatch_log_group_name 2>/dev/null); \
	if [ -z "$$LOG_GROUP_NAME" ]; then \
		echo "$(RED)❌ Log group name not found. Run \"make apply\" first to deploy infrastructure.$(RESET)" >&2; \
		exit 1; \
	fi; \
	aws logs tail "$$LOG_GROUP_NAME" --follow

clean: ## Remove tokens and backups
	@echo "$(BLUE)🧹 Cleaning up temporary files...$(RESET)"
	@rm -f .env
	@rm -f terraform.tfvars.*.backup
	@rm -f terraform.tfvars.temp
	@echo "$(GREEN)✅ Clean complete!$(RESET)"

destroy: ## Destroy infrastructure
	@echo "$(RED)🧨 WARNING: This will destroy ALL infrastructure!$(RESET)"
	@read -p "Type 'destroy' to confirm: " CONFIRM; \
	if [ "$$CONFIRM" != "destroy" ]; then \
		echo "$(RED)❌ Aborted$(RESET)"; \
		exit 1; \
	fi
	@terraform destroy -auto-approve

oauth-config: ## Display OAuth configuration for any OAuth 2.0 compliant client
	@echo "$(BLUE)🔑 OAuth Configuration Details:$(RESET)"
	@CLIENT_ID=$$(terraform output -raw entra_app_client_id 2>/dev/null); \
	TENANT_ID=$$(terraform output -raw entra_tenant_id 2>/dev/null); \
	CLIENT_SECRET=$$(terraform output -raw entra_client_secret 2>/dev/null); \
	IDENTIFIER_URI=$$(terraform output -raw entra_app_identifier_uri 2>/dev/null); \
	APP_SCOPE=$$(terraform output -raw entra_app_scope 2>/dev/null); \
	CLIENT_CREDENTIALS_SCOPE=$$(terraform output -raw entra_app_scope_client_credentials 2>/dev/null); \
	GATEWAY_URL=$$(terraform output -raw agentcore_gateway_url 2>/dev/null); \
	if [ -z "$$CLIENT_ID" ] || [ -z "$$TENANT_ID" ]; then \
		echo "$(RED)❌ Run \"make apply\" first to deploy infrastructure$(RESET)" >&2; \
		exit 1; \
	fi; \
	echo "$(GREEN)OAuth 2.0 Configuration:$(RESET)"; \
	echo "  Client ID: $$CLIENT_ID"; \
	echo "  $(RED)🚨 Client Secret: $$CLIENT_SECRET (WARNING: Treat this as a sensitive credential)$(RESET)"; \
	echo "  Authority: https://login.microsoftonline.com/$$TENANT_ID"; \
	echo "  Authorization URL: https://login.microsoftonline.com/$$TENANT_ID/oauth2/v2.0/authorize"; \
	echo "  Token URL: https://login.microsoftonline.com/$$TENANT_ID/oauth2/v2.0/token"; \
	echo "  Refresh URL: https://login.microsoftonline.com/$$TENANT_ID/oauth2/v2.0/token"; \
	echo "  Scopes:"; \
	echo "    Service-to-Service (Client Credentials): $${CLIENT_CREDENTIALS_SCOPE}"; \
	echo "    User Authentication: $${APP_SCOPE}"; \
	echo "    Microsoft 365 Copilot Integration: $${CLIENT_CREDENTIALS_SCOPE}"; \
	echo "  Additional Scopes: User.Read, openid, profile, email"; \
	echo ""; \
	echo "$(YELLOW)Note: The identifier URI (api://$$CLIENT_ID) is for reference only and not used in modern Azure AD authentication.$(RESET)"; \
	echo ""; \
	echo "$(GREEN)MCP Inspector Configuration:$(RESET)"; \
	if [ -n "$$GATEWAY_URL" ] && [ "$$GATEWAY_URL" != "null" ]; then \
		echo "  Gateway URL (streamable): $$GATEWAY_URL"; \
	else \
		echo "  ❌ Gateway not yet deployed. Deploy first with '\''make deploy'\''"; \
	fi; \
	echo ""; \
	echo "$(YELLOW)For service principal authentication:$(RESET)"; \
	echo "  1. Use the Client ID and Client Secret for authentication"; \
	echo "  2. Include the scopes in your token request"; \
	echo ""; \
	echo "$(YELLOW)Note: If configuring with a non-Dynamic Client Registration compatible OAuth provider, you may need to run '\''make add-redirect-url'\'' to manually add redirect URIs.$(RESET)"; \
	echo "";

add-redirect-url: ## Add custom OAuth redirect URL to the Entra ID application
	@CLIENT_ID=$$(terraform output -raw entra_app_client_id 2>/dev/null); \
	if [ -z "$$CLIENT_ID" ]; then \
		echo "❌ Run \"make deploy\" first to deploy infrastructure"; \
		exit 1; \
	fi; \
	echo "🔗 Adding redirect URL to Entra ID application..."; \
	read -p "Enter redirect URI to add: " REDIRECT_URI; \
	if [ -z "$$REDIRECT_URI" ]; then \
		echo "❌ Redirect URI cannot be empty"; \
		exit 1; \
	fi; \
	echo "📝 Adding $$REDIRECT_URI to application $$CLIENT_ID..."; \
	CURRENT_URIS=$$(az ad app show --id $$CLIENT_ID --query "web.redirectUris" -o tsv 2>/dev/null); \
	if [ -z "$$CURRENT_URIS" ]; then \
		az ad app update --id $$CLIENT_ID --web-redirect-uris "$$REDIRECT_URI" >/dev/null 2>&1; \
	else \
		az ad app update --id $$CLIENT_ID --web-redirect-uris $$CURRENT_URIS "$$REDIRECT_URI" >/dev/null 2>&1; \
	fi; \
	echo "✅ Redirect URI added successfully!"; \
	echo "💡 Due to ignore_changes, this URI will persist through future Terraform applies."; \
	echo ""; \
	echo "📌 Alternative methods:"; \
	echo "   • Azure Portal: App Registrations → $$CLIENT_ID → Authentication → Add URI"; \
	echo "   • Azure CLI: az ad app update --id $$CLIENT_ID --web-redirect-uris YOUR_URI"; \
	echo "   • Note: All methods persist due to Terraform ignore_changes lifecycle setting.";

remove-redirect-url: ## Remove custom OAuth redirect URL from the Entra ID application
	@CLIENT_ID=$$(terraform output -raw entra_app_client_id 2>/dev/null); \
	if [ -z "$$CLIENT_ID" ]; then \
		echo "❌ Run \"make deploy\" first to deploy infrastructure"; \
		exit 1; \
	fi; \
	echo "🔗 Removing redirect URL from Entra ID application..."; \
	read -p "Enter redirect URI to remove: " REDIRECT_URI; \
	if [ -z "$$REDIRECT_URI" ]; then \
		echo "❌ Redirect URI cannot be empty"; \
		exit 1; \
	fi; \
	echo "📝 Removing $$REDIRECT_URI from application $$CLIENT_ID..."; \
	CURRENT_URIS=$$(az ad app show --id $$CLIENT_ID --query "web.redirectUris" -o tsv 2>/dev/null); \
	if [ -n "$$CURRENT_URIS" ]; then \
		REMAINING_URIS=$$(echo "$$CURRENT_URIS" | tr '\t' ' ' | xargs -n1 | grep -v "^$$REDIRECT_URI$$" | xargs); \
		az ad app update --id $$CLIENT_ID --web-redirect-uris $$REMAINING_URIS >/dev/null 2>&1; \
	fi; \
	echo "✅ Redirect URI removed successfully!"; \
	echo "💡 Due to ignore_changes, this change will persist through future Terraform applies."; \
	echo ""; \
	echo "📌 Alternative methods:"; \
	echo "   • Azure Portal: App Registrations → $$CLIENT_ID → Authentication → Remove URI"; \
	echo "   • Azure CLI: az ad app update --id $$CLIENT_ID --web-redirect-uris NEW_LIST"; \
	echo "   • Note: All methods persist due to Terraform ignore_changes lifecycle setting.";