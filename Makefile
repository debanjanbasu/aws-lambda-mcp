.PHONY: help check-tools schema build release test all deploy tf-init tf-plan tf-apply tf-destroy login test-token test-lambda logs clean kill-inspector oauth-config add-redirect-url setup-backend update-secrets

AWS_REGION ?= ap-southeast-2

# Colors for output
RED := \033[1;31m
GREEN := \033[1;32m
YELLOW := \033[1;33m
BLUE := \033[1;34m
CYAN := \033[1;36m
BOLD := \033[1m
RESET := \033[0m

help: ## ✨ Show this help
	@echo "$(CYAN)$(BOLD)AWS Lambda MCP - Developer Commands$(RESET)"
	@echo ""
	@echo "$(GREEN)Build & Test:$(RESET)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '^(check-tools|schema|build|release|test|all|update-deps):' | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Deployment:$(RESET)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '^(check-backend-config|setup-backend|deploy|tf-destroy):' | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Development Tools:$(RESET)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '^(login|test-token|test-lambda|logs|clean|kill-inspector|oauth-config|add-redirect-url|remove-redirect-url|update-secrets):' | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Terraform Commands:$(RESET)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '^(tf-init|tf-plan|tf-apply):' | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)For full infrastructure commands:$(RESET) $(YELLOW)cd iac && make help$(RESET)"

# Tool Prerequisites Check
check-tools:
	@echo "$(BLUE)🔧 Checking required tools...$(RESET)"
	@if [ -z "$$CI" ]; then \
		command -v cargo >/dev/null 2>&1 || (echo "$(RED)❌ cargo not found. Installing Rust nightly...$(RESET)" && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly -y && source $$HOME/.cargo/env && rustup component add rust-src && rustup target add aarch64-unknown-linux-gnu && echo "$(GREEN)✅ Rust nightly installed$(RESET)"); \
		command -v zig >/dev/null 2>&1 || ( \
			echo "$(BLUE)📦 Installing Zig...$(RESET)" && \
			if command -v brew >/dev/null 2>&1; then \
				brew install zig; \
			elif command -v apt >/dev/null 2>&1; then \
				sudo apt update && sudo apt install -y zig; \
			else \
				echo "$(BLUE)📦 Downloading Zig...$(RESET)" && \
				curl -L https://ziglang.org/download/latest/zig-linux-x86_64.tar.xz | tar -xJ -C /tmp && \
				sudo mv /tmp/zig-linux-x86_64*/zig /usr/local/bin/ && \
				sudo mv /tmp/zig-linux-x86_64*/lib /usr/local/lib/zig && \
				rm -rf /tmp/zig-linux-x86_64*; \
			fi && \
			echo "$(GREEN)✅ Zig installed$(RESET)" \
		); \
		command -v cargo-lambda >/dev/null 2>&1 || (echo "$(BLUE)📦 Installing cargo-lambda...$(RESET)" && cargo install cargo-lambda && echo "$(GREEN)✅ cargo-lambda installed$(RESET)"); \
		command -v upx >/dev/null 2>&1 || ( \
			echo "$(BLUE)📦 Installing UPX...$(RESET)" && \
			if command -v brew >/dev/null 2>&1; then \
				brew install upx; \
			elif command -v apt >/dev/null 2>&1; then \
				sudo apt update && sudo apt install -y upx-ucl; \
			else \
				echo "$(RED)❌ UPX not found and no package manager detected. Install manually: brew install upx (macOS) or apt install upx-ucl (Linux)$(RESET)" && exit 1; \
			fi && \
			echo "$(GREEN)✅ UPX installed$(RESET)" \
		); \
		command -v jq >/dev/null 2>&1 || ( \
			echo "$(BLUE)📦 Installing jq...$(RESET)" && \
			if command -v brew >/dev/null 2>&1; then \
				brew install jq; \
			elif command -v apt >/dev/null 2>&1; then \
				sudo apt update && sudo apt install -y jq; \
			else \
				echo "$(RED)❌ jq not found and no package manager detected. Install manually: brew install jq (macOS) or apt install jq (Linux)$(RESET)" && exit 1; \
			fi && \
			echo "$(GREEN)✅ jq installed$(RESET)" \
		); \
		command -v terraform >/dev/null 2>&1 || ( \
			echo "$(BLUE)📦 Downloading Terraform...$(RESET)" && \
			curl -fsSL https://releases.hashicorp.com/terraform/1.9.8/terraform_1.9.8_linux_arm64.zip -o /tmp/terraform.zip && \
			unzip -o /tmp/terraform.zip -d /tmp && \
			sudo mv /tmp/terraform /usr/local/bin/terraform && \
			sudo chmod +x /usr/local/bin/terraform && \
			rm /tmp/terraform.zip && \
			echo "$(GREEN)✅ Terraform installed$(RESET)" \
		); \
	else \
		echo "$(YELLOW)⚠️  Skipping tool installation (in CI). Tools installed by workflow.$(RESET)"; \
	fi
	@echo "$(GREEN)✅ All required tools ready$(RESET)"

# Smart Backend Configuration Check
check-backend-config:
	@if [ ! -f iac/backend.config ]; then \
		echo "$(YELLOW)⚠️  backend.config file not found!$(RESET)"; \
		echo ""; \
		echo "You need to run the one-time backend setup first:"; \
		echo "  $(CYAN)make setup-backend$(RESET)"; \
		echo ""; \
		echo "This will:"; \
		echo "  1. Create an S3 bucket for Terraform state"; \
		echo "  2. Enable native S3 state locking (Terraform 1.10+)"; \
		echo "  3. Generate the iac/backend.config file"; \
		echo ""; \
		echo "After setup, run '$(CYAN)make tf-init$(RESET)' to initialize Terraform."; \
		exit 1; \
	else \
		echo "$(GREEN)✅ backend.config file exists$(RESET)"; \
	fi

# Build Commands
schema: ## 📄 Generate tool_schema.json
	@echo "$(BLUE)📄 Generating tool schemas...$(RESET)"
	@cargo run --bin generate-schema --features schema-gen --color=always

build: schema ## 🐳 Build Lambda (debug)
	@echo "$(BLUE)🔨 Building debug version...$(RESET)"
	@cargo lambda build --bin aws-lambda-mcp --color=always

release: schema check-tools ## 📦 Build Lambda (release, ARM64) with UPX compression
	@echo "$(BLUE)🚀 Building release version (ARM64 + UPX)...$(RESET)"
	@cargo lambda build --release --arm64 --bin aws-lambda-mcp --color=always
	@echo "$(BLUE)📦 Compressing binary with UPX (--best --lzma)...$(RESET)"
	@upx --best --lzma target/lambda/aws-lambda-mcp/bootstrap
	@echo "$(GREEN)📊 Final size:$(RESET)"
	@ls -lh target/lambda/aws-lambda-mcp/bootstrap

test: ## 🧪 Run tests
	@echo "$(BLUE)🧪 Running tests...$(RESET)"
	@cargo test --color=always

update-deps: ## ⬆️ Update all dependencies to their latest versions
	@echo "$(BLUE)📦 Updating dependencies...$(RESET)"
	@cargo update
	@cd iac && terraform init -upgrade
	@echo "$(GREEN)✅ Dependencies updated!$(RESET)"

all: test release ## ✨ Run tests and build release

# Deployment Commands (Smart - checks backend config)
deploy: ## 🚀 Build and deploy to AWS (requires backend config)
	@make check-backend-config
	@echo "$(BLUE)🚀 Building and deploying to AWS...$(RESET)"
	@make release
	@cd iac && $(MAKE) deploy

tf-init: ## ⚙️ Initialize Terraform (requires backend config)
	@make check-backend-config
	@echo "$(BLUE)⚙️  Initializing Terraform...$(RESET)"
	@cd iac && terraform init -backend-config=backend.config

tf-plan: release ## 📋 Plan Terraform changes (builds Lambda first, requires backend config)
	@make check-backend-config
	@echo "$(BLUE)📋 Planning Terraform deployment...$(RESET)"
	@cd iac && terraform plan

tf-apply: release ## 🚀 Apply Terraform changes (builds Lambda first, requires backend config)
	@make check-backend-config
	@echo "$(BLUE)🚀 Applying Terraform deployment...$(RESET)"
	@cd iac && terraform apply -auto-approve

tf-destroy: ## 🧨 Destroy Terraform resources (requires backend config)
	@make check-backend-config
	@echo "$(YELLOW)🧨 Destroying Terraform resources...$(RESET)"
	@cd iac && terraform destroy -auto-approve

# Infrastructure Commands
setup-backend: ## ⚙️ Create S3 backend for Terraform state (native locking)
	@echo "$(BLUE)⚙️  Setting up Terraform backend...$(RESET)"
	@command -v aws >/dev/null 2>&1 || (echo "$(RED)❌ AWS CLI not found. Install: https://aws.amazon.com/cli/$(RESET)" && exit 1)
	@aws sts get-caller-identity >/dev/null 2>&1 || (echo "$(RED)❌ AWS CLI not configured. Run: aws configure$(RESET)" && exit 1)
	@read -p "Enter a globally unique S3 bucket name for Terraform state: " BUCKET_NAME; \
	if [ -z "$$BUCKET_NAME" ]; then \
		echo "$(RED)❌ Bucket name cannot be empty.$(RESET)"; \
		exit 1; \
	fi; \
	echo "$(BLUE)▶️ Creating S3 bucket '$$BUCKET_NAME' in region $(AWS_REGION)...$(RESET)"; \
	aws s3api create-bucket --bucket $$BUCKET_NAME --region $(AWS_REGION) --create-bucket-configuration LocationConstraint=$(AWS_REGION) > /dev/null; \
	echo "$(BLUE)▶️ Enabling versioning and encryption for '$$BUCKET_NAME'...$(RESET)"; \
	aws s3api put-bucket-versioning --bucket $$BUCKET_NAME --versioning-configuration Status=Enabled > /dev/null; \
	aws s3api put-bucket-encryption --bucket $$BUCKET_NAME --server-side-encryption-configuration '{"Rules": [{"ApplyServerSideEncryptionByDefault": {"SSEAlgorithm": "AES256"}}]}' > /dev/null; \
	echo "$(BLUE)▶️ Creating 'iac/backend.config' for local use...$(RESET)"; \
	echo "bucket         = \"$$BUCKET_NAME\"" > iac/backend.config; \
	echo "key            = \"aws-lambda-mcp/terraform.tfstate\"" >> iac/backend.config; \
	echo "region         = \"$(AWS_REGION)\"" >> iac/backend.config; \
	echo "use_lockfile   = true" >> iac/backend.config; \
	echo "$(GREEN)✅ Backend setup complete!$(RESET)"; \
	echo "$(CYAN)ℹ️  Using native S3 state locking (Terraform 1.10+)$(RESET)"; \
	echo "Run '$(CYAN)make tf-init$(RESET)' to initialize Terraform with the new backend."; \
	echo "TF_BACKEND_BUCKET=\"$$BUCKET_NAME\"" >> .env

login: ## 🔑 Authenticate AWS + Azure CLIs
	@echo "$(BLUE)🔐 Authenticating AWS + Azure CLIs...$(RESET)"
	@cd iac && $(MAKE) login

test-token: ## 🔑 Get OAuth token + launch MCP Inspector
	@echo "$(BLUE)🔑 Getting OAuth token...$(RESET)"
	@lsof -ti:6274,6277 2>/dev/null | xargs kill -9 2>/dev/null || true
	@cd iac && $(MAKE) test-token

test-lambda: ## 🧪 Test Lambda directly (bypass Gateway)
	@echo "$(BLUE)🧪 Testing Lambda directly...$(RESET)"
	@cd iac && $(MAKE) test-lambda

logs: ## 📜 Tail Lambda logs
	@echo "$(BLUE)📜 Tailing Lambda logs (Ctrl+C to exit)...$(RESET)"; \
	@cd iac && $(MAKE) logs

clean: ## 🧹 Remove tokens and backups
	@echo "$(BLUE)🧹 Cleaning up...$(RESET)"
	@cd iac && $(MAKE) clean

kill-inspector: ## 🛑 Kill any running MCP Inspector processes
	@echo "$(BLUE)🛑 Killing MCP Inspector processes...$(RESET)"
	@lsof -ti:6274,6277 2>/dev/null | xargs kill -9 2>/dev/null && echo "$(GREEN)✅ Killed MCP Inspector processes$(RESET)" || echo "$(YELLOW)No MCP Inspector processes running$(RESET)"

oauth-config: ## 📋 Display OAuth configuration for any OAuth 2.0 compliant client
	@echo "$(BLUE)🔑 Displaying OAuth configuration...$(RESET)"
	@cd iac && $(MAKE) oauth-config

add-redirect-url: ## 🔗 Add custom OAuth redirect URL to terraform.tfvars
	@echo "$(BLUE)🔗 Adding redirect URL to Entra ID app...$(RESET)"
	@cd iac && $(MAKE) add-redirect-url

remove-redirect-url: ## 🔗 Remove custom OAuth redirect URL from terraform.tfvars
	@echo "$(BLUE)🔗 Removing redirect URL from Entra ID app...$(RESET)"
	@cd iac && $(MAKE) remove-redirect-url

update-secrets: ## 🔐 Update GitHub repository secrets from a .env file (for GitHub Actions and Dependabot)
	@echo "$(BLUE)🔐 Updating GitHub repository secrets from .env file...$(RESET)"
	@if [ ! -f .env ]; then \
		echo "$(RED)❌ .env file not found! Create a .env file with your secrets (e.g., MY_SECRET=value).$(RESET)"; \
		exit 1; \
	fi
	@echo "$(BLUE)Setting secrets for GitHub Actions...$(RESET)"
	@gh secret set -f .env --app actions
	@echo "$(BLUE)Setting secrets for Dependabot...$(RESET)"
	@gh secret set -f .env --app dependabot
	@echo "$(GREEN)✅ GitHub secrets updated for both GitHub Actions and Dependabot!$(RESET)"

.DEFAULT_GOAL := help