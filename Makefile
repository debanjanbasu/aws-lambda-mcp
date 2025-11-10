.PHONY: help schema build release test all deploy tf-init tf-plan tf-apply tf-destroy login test-token test-lambda logs clean kill-inspector oauth-config add-redirect-url setup-backend

AWS_REGION ?= ap-southeast-2

help: ## Show this help
	@echo "Root Makefile - Lambda Build & Infrastructure"
	@echo ""
	@echo "Build Commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '^(schema|build|release|test|all):' | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Infrastructure Commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '^(login|setup-backend|deploy|tf-|test-token|test-lambda|logs|clean|kill-inspector|oauth-config|add-redirect-url):' | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "For more iac commands: cd iac && make help"

schema: ## Generate tool_schema.json
	@cargo run --bin generate-schema --features schema-gen --color=always

build: schema ## Build Lambda (debug)
	@cargo lambda build --bin aws-lambda-mcp --color=always

release: schema ## Build Lambda (release, ARM64) with UPX compression
	@cargo lambda build --release --arm64 --bin aws-lambda-mcp --color=always
	@echo "Compressing binary with UPX (--best --lzma)..."
	@upx --best --lzma target/lambda/aws-lambda-mcp/bootstrap
	@echo "Final size:"
	@ls -lh target/lambda/aws-lambda-mcp/bootstrap

test: ## Run tests
	@cargo test --color=always

all: test release ## Run tests and build release

# Infrastructure commands
login: ## Authenticate AWS + Azure CLIs
	@cd iac && $(MAKE) login

setup-backend: ## Create S3/DynamoDB backend for Terraform state
	@read -p "Enter a globally unique S3 bucket name for Terraform state: " BUCKET_NAME; \
	if [ -z "$$BUCKET_NAME" ]; then \
		echo "❌ Bucket name cannot be empty."; \
		exit 1; \
	fi; \
	DYNAMODB_TABLE="terraform-state-lock-mcp"; \
	echo "▶️ Creating S3 bucket '$$BUCKET_NAME' in region $(AWS_REGION)..."; \
	aws s3api create-bucket --bucket $$BUCKET_NAME --region $(AWS_REGION) --create-bucket-configuration LocationConstraint=$(AWS_REGION) > /dev/null; \
	echo "▶️ Enabling versioning and encryption for '$$BUCKET_NAME'..."; \
	aws s3api put-bucket-versioning --bucket $$BUCKET_NAME --versioning-configuration Status=Enabled > /dev/null; \
	aws s3api put-bucket-encryption --bucket $$BUCKET_NAME --server-side-encryption-configuration '{"Rules": [{"ApplyServerSideEncryptionByDefault": {"SSEAlgorithm": "AES256"}}]}' > /dev/null; \
	echo "▶️ Creating DynamoDB table '$$DYNAMODB_TABLE' for state locking..."; \
	aws dynamodb create-table \
		--table-name $$DYNAMODB_TABLE \
		--attribute-definitions AttributeName=LockID,AttributeType=S \
		--key-schema AttributeName=LockID,KeyType=HASH \
		--provisioned-throughput ReadCapacityUnits=1,WriteCapacityUnits=1 \
		--region $(AWS_REGION) > /dev/null || echo "⚠️ DynamoDB table may already exist. That's okay."; \
	echo "▶️ Creating 'iac/backend.config' for local use..."; \
	echo "bucket         = \"$$BUCKET_NAME\"" > iac/backend.config; \
	echo "key            = \"aws-lambda-mcp/terraform.tfstate\"" >> iac/backend.config; \
	echo "region         = \"$(AWS_REGION)\"" >> iac/backend.config; \
	echo "dynamodb_table = \"$$DYNAMODB_TABLE\"" >> iac/backend.config; \
	echo "✅ Backend setup complete!"; \
	echo "Run 'make tf-init' to initialize Terraform with the new backend."

tf-init: ## Initialize Terraform
	@echo "Initializing Terraform..."
	@cd iac && terraform init -backend-config=backend.config

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
		elif command -v xclip >/dev/null 2>&1; then \			echo "$$CLIENT_SECRET" | xclip -selection clipboard; \
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
	cd iac && \
	TFVARS_FILE="terraform.tfvars"; \
	TEMP_TFVARS_FILE="terraform.tfvars.tmp"; \
	REDIRECT_URI_ESCAPED=$$(echo "$$REDIRECT_URL" | sed "s|\"|\\\"|g"); \
	\
	if ! grep -q "entra_redirect_uris" "$$TFVARS_FILE" 2>/dev/null; then \
		echo "entra_redirect_uris = [" >> "$$TFVARS_FILE"; \
		echo "  \"$$REDIRECT_URI_ESCAPED\"" >> "$$TFVARS_FILE"; \
		echo "]" >> "$$TFVARS_FILE"; \
	else \
		if grep -q "\"$$REDIRECT_URI_ESCAPED\"" "$$TFVARS_FILE" 2>/dev/null; then \
			echo "⚠️  URL already exists in terraform.tfvars"; \
			exit 0; \
		fi; \
		awk -v new_uri="  \"$$REDIRECT_URI_ESCAPED\"," '/^]/ && !x { print new_uri; x=1 } { print }' "$$TFVARS_FILE" > "$$TEMP_TFVARS_FILE"; \
		mv "$$TEMP_TFVARS_FILE" "$$TFVARS_FILE"; \
	fi; \
	echo "✅ Added to terraform.tfvars"; \
	echo ""; \
	echo "Applying changes..."; \
	terraform apply -auto-approve;
tf-destroy: ## Destroy Terraform resources
	@echo "Destroying Terraform resources..."
	@cd iac && terraform destroy -auto-approve

.DEFAULT_GOAL := help
