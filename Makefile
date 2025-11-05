.PHONY: help schema build release test all deploy tf-init tf-plan tf-apply tf-destroy login test-token test-lambda logs clean kill-inspector oauth-config add-redirect-url

help: ## Show this help
	@echo "Root Makefile - Lambda Build & Infrastructure"
	@echo ""
	@echo "Build Commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '^(schema|build|release|test|all):' | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Infrastructure Commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '^(login|deploy|tf-|test-token|test-lambda|logs|clean|kill-inspector|oauth-config|add-redirect-url):' | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "For more iac commands: cd iac && make help"

schema: ## Generate tool_schema.json
	@cargo run --bin generate-schema --features schema-gen

build: schema ## Build Lambda (debug)
	@cargo lambda build --bin aws-lambda-mcp

release: schema ## Build Lambda (release, ARM64) with UPX compression
	@cargo lambda build --release --arm64 --bin aws-lambda-mcp
	@echo "Compressing binary with UPX (--best --lzma)..."
	@upx --best --lzma target/lambda/aws-lambda-mcp/bootstrap
	@echo "Final size:"
	@ls -lh target/lambda/aws-lambda-mcp/bootstrap

test: ## Run tests
	@cargo test

all: test release ## Run tests and build release

# Infrastructure commands - proxy to iac/Makefile
login: ## Authenticate AWS + Azure CLIs
	@cd iac && $(MAKE) login

tf-init: ## Initialize Terraform
	@echo "Initializing Terraform..."
	@cd iac && terraform init

tf-plan: release ## Plan Terraform changes (builds Lambda first)
	@echo "Planning Terraform deployment..."
	@cd iac && terraform plan

tf-apply: release ## Apply Terraform changes (builds Lambda first)
	@echo "Applying Terraform deployment..."
	@cd iac && terraform apply -auto-approve

deploy: tf-apply ## Build and deploy to AWS (alias for tf-apply)

test-token: ## Get OAuth token + launch MCP Inspector
	@lsof -ti:6274,6277 2>/dev/null | xargs kill -9 2>/dev/null || true
	@cd iac && $(MAKE) test-token

test-lambda: ## Test Lambda directly (bypass Gateway)
	@cd iac && $(MAKE) test-lambda

logs: ## Tail Lambda logs
	@cd iac && $(MAKE) logs

clean: ## Remove tokens and backups
	@cd iac && $(MAKE) clean

kill-inspector: ## Kill any running MCP Inspector processes
	@lsof -ti:6274,6277 2>/dev/null | xargs kill -9 2>/dev/null && echo "✅ Killed MCP Inspector processes" || echo "No MCP Inspector processes running"

oauth-config: ## Display OAuth configuration for M365 Copilot connector
	@echo "=== M365 Copilot Custom Connector - OAuth Configuration ==="
	@echo ""
	@cd iac && bash -c ' \
		CLIENT_ID=$$(terraform output -raw entra_app_client_id 2>/dev/null); \
		CLIENT_SECRET=$$(terraform output -raw entra_client_secret 2>/dev/null); \
		TENANT_ID=$$(terraform output -raw entra_tenant_id 2>/dev/null); \
		GATEWAY_URL=$$(terraform output -raw agentcore_gateway_url 2>/dev/null); \
		if [ -z "$$CLIENT_ID" ] || [ -z "$$TENANT_ID" ]; then \
			echo "❌ Error: Terraform outputs not available"; \
			echo "Run \"make deploy\" first"; \
			exit 1; \
		fi; \
		HOST=$$(echo "$$GATEWAY_URL" | sed "s|https://||" | sed "s|/mcp||"); \
		echo "📋 Copy these values to your Power Automate Custom Connector:"; \
		echo ""; \
		echo "General:"; \
		echo "  Host: $$HOST"; \
		echo "  Base URL: /mcp"; \
		echo "  Full URL: $$GATEWAY_URL"; \
		echo ""; \
		echo "Security:"; \
		echo "  Authentication type: OAuth 2.0"; \
		echo "  Identity Provider: Generic OAuth 2"; \
		echo "  Client ID: $$CLIENT_ID"; \
		echo "  Client secret: $$CLIENT_SECRET"; \
		echo "  Authorization URL: https://login.microsoftonline.com/$$TENANT_ID/oauth2/v2.0/authorize"; \
		echo "  Token URL: https://login.microsoftonline.com/$$TENANT_ID/oauth2/v2.0/token"; \
		echo "  Refresh URL: https://login.microsoftonline.com/$$TENANT_ID/oauth2/v2.0/token"; \
		echo "  Scope: api://$$CLIENT_ID/access_as_user"; \
		echo ""; \
		echo "💡 The Redirect URL will be generated when you save the connector."; \
		echo ""; \
		echo "Next step: After saving the connector, run:"; \
		echo "  make add-redirect-url"; \
		echo ""; \
		if command -v pbcopy >/dev/null 2>&1; then \
			echo "$$CLIENT_SECRET" | pbcopy; \
			echo "✅ Client secret copied to clipboard"; \
		elif command -v xclip >/dev/null 2>&1; then \
			echo "$$CLIENT_SECRET" | xclip -selection clipboard; \
			echo "✅ Client secret copied to clipboard"; \
		elif command -v xsel >/dev/null 2>&1; then \
			echo "$$CLIENT_SECRET" | xsel --clipboard --input; \
			echo "✅ Client secret copied to clipboard"; \
		elif command -v clip.exe >/dev/null 2>&1; then \
			echo "$$CLIENT_SECRET" | clip.exe; \
			echo "✅ Client secret copied to clipboard"; \
		fi; \
	'

add-redirect-url: ## Add Power Automate redirect URL to Entra ID app
	@echo "=== Add Redirect URL to Entra ID App ==="
	@echo ""
	@echo "After saving your Power Automate custom connector, copy the Redirect URL"
	@echo "from the Security tab and paste it below."
	@echo ""
	@read -p "Enter Redirect URL: " REDIRECT_URL; \
	if [ -z "$$REDIRECT_URL" ]; then \
		echo "❌ No URL provided"; \
		exit 1; \
	fi; \
	echo ""; \
	echo "Adding redirect URL: $$REDIRECT_URL"; \
	cd iac && bash -c ' \
		CURRENT_URIS=$$(grep "entra_redirect_uris" terraform.tfvars 2>/dev/null || echo ""); \
		if [ -z "$$CURRENT_URIS" ]; then \
			echo "entra_redirect_uris = [" >> terraform.tfvars; \
			echo "  \"$$REDIRECT_URL\"" >> terraform.tfvars; \
			echo "]" >> terraform.tfvars; \
		else \
			if grep -q "\"$$REDIRECT_URL\"" terraform.tfvars 2>/dev/null; then \
				echo "⚠️  URL already exists in terraform.tfvars"; \
				exit 0; \
			fi; \
			sed -i.bak "/entra_redirect_uris/a\\ \\ \"$$REDIRECT_URL\"," terraform.tfvars; \
			rm -f terraform.tfvars.bak; \
		fi; \
		echo "✅ Added to terraform.tfvars"; \
		echo ""; \
		echo "Applying changes..."; \
		terraform apply -auto-approve; \
	' 
tf-destroy: ## Destroy Terraform resources
	@echo "Destroying Terraform resources..."
	@cd iac && terraform destroy -auto-approve

.DEFAULT_GOAL := help
