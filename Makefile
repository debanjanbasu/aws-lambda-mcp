.PHONY: help schema build release test all deploy tf-init tf-plan tf-apply tf-destroy login test-token test-lambda logs clean kill-inspector oauth-config add-redirect-url setup-backend update-secrets

AWS_REGION ?= ap-southeast-2

help: ## ✨ Show this help
	@echo "\033[1;36mAWS Lambda MCP - Developer Commands\033[0m"
	@echo ""
	@echo "\033[1;32mBuild & Test:\033[0m"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '^(schema|build|release|test|all|update-deps):' | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "\033[1;32mDeployment:\033[0m"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '^(setup-backend|deploy|tf-destroy):' | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "\033[1;32mDevelopment Tools:\033[0m"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '^(test-token|test-lambda|logs|login|clean|kill-inspector|oauth-config|add-redirect-url|remove-redirect-url|update-secrets):' | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "\033[1;32mTerraform Commands:\033[0m"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '^(tf-init|tf-plan|tf-apply):' | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "\033[1;32mFor full infrastructure commands:\033[0m \033[33mcd iac && make help\033[0m"

# Smart Backend Configuration Check
check-backend-config:
	@if [ ! -f iac/backend.config ]; then \
		echo "\033[1;33m⚠️  backend.config file not found!\033[0m"; \
		echo ""; \
		echo "You need to run the one-time backend setup first:"; \
		echo "  \033[1;36mmake setup-backend\033[0m"; \
		echo ""; \
		echo "This will:"; \
		echo "  1. Create an S3 bucket for Terraform state"; \
		echo "  2. Create a DynamoDB table for state locking"; \
		echo "  3. Generate the iac/backend.config file"; \
		echo ""; \
		echo "After setup, run '\033[1;36mmake tf-init\033[0m' to initialize Terraform."; \
		exit 1; \
	else \
		echo "\033[1;32m✅ backend.config file exists\033[0m"; \
	fi

# Build Commands
schema: ## 📄 Generate tool_schema.json
	@echo "\033[1;34m📄 Generating tool schemas...\033[0m"
	@cargo run --bin generate-schema --features schema-gen --color=always

build: schema ## 🐳 Build Lambda (debug)
	@echo "\033[1;34m🔨 Building debug version...\033[0m"
	@cargo lambda build --bin aws-lambda-mcp --color=always

release: schema ## 📦 Build Lambda (release, ARM64) with UPX compression
	@echo "\033[1;34m🚀 Building release version (ARM64 + UPX)..."; \
	cargo lambda build --release --arm64 --bin aws-lambda-mcp --color=always; \
	@echo "\033[1;34m📦 Compressing binary with UPX (--best --lzma)..."; \
	upx --best --lzma target/lambda/aws-lambda-mcp/bootstrap; \
	@echo "\033[1;32m📊 Final size:\033[0m"; \
	ls -lh target/lambda/aws-lambda-mcp/bootstrap

test: ## 🧪 Run tests
	@echo "\033[1;34m🧪 Running tests...\033[0m"
	@cargo test --color=always

update-deps: ## ⬆️ Update all dependencies to their latest versions
	@echo "\033[1;34m📦 Updating dependencies...\033[0m"
	@cargo update
	@cd iac && terraform init -upgrade
	@echo "\033[1;32m✅ Dependencies updated!\033[0m"

all: test release ## ✨ Run tests and build release

# Deployment Commands (Smart - checks backend config)
deploy: ## 🚀 Build and deploy to AWS (requires backend config)
	@make check-backend-config
	@echo "\033[1;34m🚀 Building and deploying to AWS...\033[0m"
	@make release
	@cd iac && $(MAKE) deploy

tf-init: ## ⚙️ Initialize Terraform (requires backend config)
	@make check-backend-config
	@echo "\033[1;34m⚙️  Initializing Terraform...\033[0m"
	@cd iac && terraform init -backend-config=backend.config

tf-plan: release ## 📋 Plan Terraform changes (builds Lambda first, requires backend config)
	@make check-backend-config
	@echo "\033[1;34m📋 Planning Terraform deployment...\033[0m"
	@cd iac && terraform plan

tf-apply: release ## 🚀 Apply Terraform changes (builds Lambda first, requires backend config)
	@make check-backend-config
	@echo "\033[1;34m🚀 Applying Terraform deployment...\033[0m"
	@cd iac && terraform apply -auto-approve

tf-destroy: ## 🧨 Destroy Terraform resources (requires backend config)
	@make check-backend-config
	@echo "\033[1;33m🧨 Destroying Terraform resources...\033[0m"
	@cd iac && terraform destroy -auto-approve

# Infrastructure Commands
setup-backend: ## ⚙️ Create S3/DynamoDB backend for Terraform state
	@echo "\033[1;34m⚙️  Setting up Terraform backend...\033[0m"
	@read -p "Enter a globally unique S3 bucket name for Terraform state: " BUCKET_NAME; \
	if [ -z "$$BUCKET_NAME" ]; then \
		echo "\033[1;31m❌ Bucket name cannot be empty.\033[0m"; \
		exit 1; \
	fi; \
	DYNAMODB_TABLE="terraform-state-lock-mcp"; \
	echo "\033[1;34m▶️ Creating S3 bucket '$$BUCKET_NAME' in region $(AWS_REGION)..."; \
	aws s3api create-bucket --bucket $$BUCKET_NAME --region $(AWS_REGION) --create-bucket-configuration LocationConstraint=$(AWS_REGION) > /dev/null; \
	echo "\033[1;34m▶️ Enabling versioning and encryption for '$$BUCKET_NAME'..."; \
	aws s3api put-bucket-versioning --bucket $$BUCKET_NAME --versioning-configuration Status=Enabled > /dev/null; \
	aws s3api put-bucket-encryption --bucket $$BUCKET_NAME --server-side-encryption-configuration '{"Rules": [{"ApplyServerSideEncryptionByDefault": {"SSEAlgorithm": "AES256"}}]}' > /dev/null; \
	echo "\033[1;34m▶️ Creating DynamoDB table '$$DYNAMODB_TABLE' for state locking..."; \
	aws dynamodb create-table \
		--table-name $$DYNAMODB_TABLE \
		--attribute-definitions AttributeName=LockID,AttributeType=S \
		--key-schema AttributeName=LockID,KeyType=HASH \
		--provisioned-throughput ReadCapacityUnits=1,WriteCapacityUnits=1 \
		--region $(AWS_REGION) > /dev/null || echo "\033[1;33m⚠️ DynamoDB table may already exist. That's okay.\033[0m"; \
	echo "\033[1;34m▶️ Creating 'iac/backend.config' for local use...\033[0m"; \
	echo "bucket         = \"$$BUCKET_NAME\"" > iac/backend.config; \
	echo "key            = \"aws-lambda-mcp/terraform.tfstate\"" >> iac/backend.config; \
	echo "region         = \"$(AWS_REGION)\"" >> iac/backend.config; \
	echo "dynamodb_table = \"$$DYNAMODB_TABLE\"" >> iac/backend.config; \
	echo "\033[1;32m✅ Backend setup complete!\033[0m"; \
	echo "Run '\033[1;36mmake tf-init\033[0m' to initialize Terraform with the new backend."; \
	echo "TF_BACKEND_BUCKET=\"$$BUCKET_NAME\"" >> .env; \
	echo "TF_BACKEND_DYNAMODB_TABLE=\"$$DYNAMODB_TABLE\"" >> .env

login: ## 🔑 Authenticate AWS + Azure CLIs
	@echo "\033[1;34m🔐 Authenticating AWS + Azure CLIs...\033[0m"
	@cd iac && $(MAKE) login

test-token: ## 🔑 Get OAuth token + launch MCP Inspector
	@echo "\033[1;34m🔑 Getting OAuth token...\033[0m"
	@lsof -ti:6274,6277 2>/dev/null | xargs kill -9 2>/dev/null || true
	@cd iac && $(MAKE) test-token

test-lambda: ## 🧪 Test Lambda directly (bypass Gateway)
	@echo "\033[1;34m🧪 Testing Lambda directly...\033[0m"
	@cd iac && $(MAKE) test-lambda

logs: ## 📜 Tail Lambda logs
	@echo "\033[1;34m📜 Tailing Lambda logs (Ctrl+C to exit)..."; \
	@cd iac && $(MAKE) logs

clean: ## 🧹 Remove tokens and backups
	@echo "\033[1;34m🧹 Cleaning up...\033[0m"
	@cd iac && $(MAKE) clean

kill-inspector: ## 🛑 Kill any running MCP Inspector processes
	@echo "\033[1;34m🛑 Killing MCP Inspector processes...\033[0m"
	@lsof -ti:6274,6277 2>/dev/null | xargs kill -9 2>/dev/null && echo "\033[1;32m✅ Killed MCP Inspector processes\033[0m" || echo "\033[1;33mNo MCP Inspector processes running\033[0m"

oauth-config: ## 📋 Display OAuth configuration for any OAuth 2.0 compliant client
	@echo "\033[1;34m🔑 Displaying OAuth configuration...\033[0m"
	@cd iac && $(MAKE) oauth-config

add-redirect-url: ## 🔗 Add custom OAuth redirect URL to terraform.tfvars
	@echo "\033[1;34m🔗 Adding redirect URL to Entra ID app...\033[0m"
	@cd iac && $(MAKE) add-redirect-url

remove-redirect-url: ## 🔗 Remove custom OAuth redirect URL from terraform.tfvars
	@echo "\033[1;34m🔗 Removing redirect URL from Entra ID app...\033[0m"
	@cd iac && $(MAKE) remove-redirect-url

update-secrets: ## 🔐 Update GitHub repository secrets from a .env file (for GitHub Actions and Dependabot)
	@echo "\033[1;34m🔐 Updating GitHub repository secrets from .env file...\033[0m"
	@if [ ! -f .env ]; then \
		echo "\033[1;31m❌ .env file not found! Create a .env file with your secrets (e.g., MY_SECRET=value).\033[0m"; \
		exit 1; \
	fi
	@echo "\033[1;34mSetting secrets for GitHub Actions...\033[0m"
	@gh secret set -f .env --app actions
	@echo "\033[1;34mSetting secrets for Dependabot...\033[0m"
	@gh secret set -f .env --app dependabot
	@echo "\033[1;32m✅ GitHub secrets updated for both GitHub Actions and Dependabot!\033[0m"

.DEFAULT_GOAL := help