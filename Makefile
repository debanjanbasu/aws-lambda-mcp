.PHONY: help check-tools schema build release test all deploy tf-init tf-plan tf-apply tf-destroy login test-token test-lambda logs clean kill-inspector oauth-config add-redirect-url remove-redirect-url setup-backend update-secrets

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
		command -v cargo >/dev/null 2>&1 || ( \
			echo "$(BLUE)📦 Installing Rust nightly...$(RESET)" && \
			curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly -y && \
			source $$HOME/.cargo/env && \
			rustup component add rust-src && \
			rustup target add aarch64-unknown-linux-gnu && \
			echo "$(GREEN)✅ Rust nightly installed$(RESET)" \
		); \
		command -v zig >/dev/null 2>&1 || ( \
			echo "$(BLUE)📦 Installing Zig...$(RESET)" && \
			if command -v brew >/dev/null 2>&1; then \
				brew install zig && echo "$(GREEN)✅ Zig installed via Homebrew$(RESET)"; \
			elif command -v apt >/dev/null 2>&1; then \
				sudo apt update && sudo apt install -y zig && echo "$(GREEN)✅ Zig installed via APT$(RESET)"; \
			else \
				echo "$(RED)❌ Zig installation requires a package manager (Homebrew or APT)$(RESET)"; \
				echo "$(YELLOW)Please install Zig manually:$(RESET)"; \
				echo "$(YELLOW)  macOS: brew install zig$(RESET)"; \
				echo "$(YELLOW)  Linux: sudo apt install zig (or see https://ziglang.org/download/)$(RESET)"; \
				exit 1; \
			fi \
		); \
		command -v cargo-lambda >/dev/null 2>&1 || ( \
			echo "$(BLUE)📦 Installing cargo-lambda...$(RESET)" && \
			if command -v brew >/dev/null 2>&1; then \
				brew install cargo-lambda && echo "$(GREEN)✅ cargo-lambda installed via Homebrew$(RESET)"; \
			else \
				cargo install cargo-lambda && echo "$(GREEN)✅ cargo-lambda installed via Cargo$(RESET)"; \
			fi \
		); \
		command -v upx >/dev/null 2>&1 || ( \
			echo "$(BLUE)📦 Installing UPX...$(RESET)" && \
			if command -v brew >/dev/null 2>&1; then \
				brew install upx && echo "$(GREEN)✅ UPX installed via Homebrew$(RESET)"; \
			elif command -v apt >/dev/null 2>&1; then \
				sudo apt update && sudo apt install -y upx-ucl && echo "$(GREEN)✅ UPX installed via APT$(RESET)"; \
			else \
				echo "$(RED)❌ UPX not found and no package manager detected. Install manually: brew install upx (macOS) or apt install upx-ucl (Linux)$(RESET)" && exit 1; \
			fi \
		); \
		command -v jq >/dev/null 2>&1 || ( \
			echo "$(BLUE)📦 Installing jq...$(RESET)" && \
			if command -v brew >/dev/null 2>&1; then \
				brew install jq && echo "$(GREEN)✅ jq installed via Homebrew$(RESET)"; \
			elif command -v apt >/dev/null 2>&1; then \
				sudo apt update && sudo apt install -y jq && echo "$(GREEN)✅ jq installed via APT$(RESET)"; \
			else \
				echo "$(RED)❌ jq not found and no package manager detected. Install manually: brew install jq (macOS) or apt install jq (Linux)$(RESET)" && exit 1; \
			fi \
		); \
		command -v terraform >/dev/null 2>&1 || ( \
			echo "$(BLUE)📦 Installing Terraform...$(RESET)" && \
			if command -v brew >/dev/null 2>&1; then \
				brew tap hashicorp/tap && \
				brew install hashicorp/tap/terraform && \
				echo "$(GREEN)✅ Terraform installed via Homebrew$(RESET)"; \
			elif command -v apt-get >/dev/null 2>&1; then \
				echo "$(BLUE)📦 Using APT package manager...$(RESET)" && \
				sudo apt-get update && \
				sudo apt-get install -y gnupg software-properties-common && \
				wget -qO- https://apt.releases.hashicorp.com/gpg | gpg --dearmor | sudo tee /usr/share/keyrings/hashicorp-archive-keyring.gpg > /dev/null && \
				echo "deb [signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] https://apt.releases.hashicorp.com $$(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/hashicorp.list && \
				sudo apt-get update && \
				sudo apt-get install -y terraform && \
				echo "$(GREEN)✅ Terraform installed via APT$(RESET)"; \
			elif command -v yum >/dev/null 2>&1; then \
				echo "$(BLUE)📦 Using YUM package manager...$(RESET)" && \
				sudo yum install -y yum-utils && \
				sudo yum-config-manager --add-repo https://rpm.releases.hashicorp.com/RHEL/hashicorp.repo && \
				sudo yum -y install terraform && \
				echo "$(GREEN)✅ Terraform installed via YUM$(RESET)"; \
			elif command -v dnf >/dev/null 2>&1; then \
				echo "$(BLUE)📦 Using DNF package manager...$(RESET)" && \
				sudo dnf install -y dnf-plugins-core && \
				sudo dnf config-manager addrepo --from-repofile=https://rpm.releases.hashicorp.com/fedora/hashicorp.repo && \
				sudo dnf -y install terraform && \
				echo "$(GREEN)✅ Terraform installed via DNF$(RESET)"; \
			else \
				echo "$(RED)❌ Terraform installation requires a package manager (Homebrew, APT, YUM, or DNF)$(RESET)"; \
				echo "$(YELLOW)Please install Terraform manually:$(RESET)"; \
				echo "$(YELLOW)  macOS: brew tap hashicorp/tap && brew install hashicorp/tap/terraform$(RESET)"; \
				echo "$(YELLOW)  Ubuntu/Debian: https://developer.hashicorp.com/terraform/tutorials/aws-get-started/install-cli$(RESET)"; \
				echo "$(YELLOW)  RHEL/CentOS/Fedora: https://developer.hashicorp.com/terraform/tutorials/aws-get-started/install-cli$(RESET)"; \
				exit 1; \
			fi \
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
	@cargo lambda build --bin interceptor --color=always

release: schema check-tools ## 📦 Build Lambda (release, ARM64) with UPX compression
	@echo "$(BLUE)🚀 Building release version (ARM64)...$(RESET)"
	@cargo lambda build --release --arm64 --bin aws-lambda-mcp --color=always
	@cargo lambda build --release --arm64 --bin interceptor --color=always
	@echo "$(BLUE)🗜️  Compressing binaries with UPX (--best --lzma)...$(RESET)"
	@upx --best --lzma target/lambda/aws-lambda-mcp/bootstrap
	@upx --best --lzma target/lambda/interceptor/bootstrap
	@echo "$(GREEN)📊 Final binary sizes:$(RESET)"
	@ls -lh target/lambda/aws-lambda-mcp/bootstrap target/lambda/interceptor/bootstrap

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
	@make schema
	@echo "$(YELLOW)🧨 Destroying Terraform resources...$(RESET)"
	@echo "$(BLUE)📦 Creating dummy bootstrap files for destroy...$(RESET)"
	@mkdir -p target/lambda/aws-lambda-mcp target/lambda/interceptor
	@touch target/lambda/aws-lambda-mcp/bootstrap target/lambda/interceptor/bootstrap
	@cd iac && terraform destroy -auto-approve

# Infrastructure Commands
setup-backend: ## ⚙️ Create S3 backend for Terraform state (native locking)
	@PROCEED=true; \
	echo "$(BLUE)⚙️  Setting up Terraform backend...$(RESET)"; \
	if [ -f iac/backend.config ]; then \
		echo "$(YELLOW)⚠️  A backend configuration already exists:$(RESET)"; \
		echo ""; \
		cat iac/backend.config | sed "s/^/  /"; \
		echo ""; \
		echo "$(CYAN)💡 Don't worry! Your existing config will be automatically backed up.$(RESET)"; \
		echo ""; \
		echo -n "Do you want to proceed and create a new backend? (y/N): "; \
		read CONFIRM; \
		if [ "$$CONFIRM" != "y" ] && [ "$$CONFIRM" != "Y" ]; then \
			echo "$(GREEN)✅ Aborted. Existing backend preserved.$(RESET)"; \
			PROCEED=false; \
		else \
			BACKUP_FILE="iac/backend.config.backup.$$(date +%Y%m%d_%H%M%S)"; \
			cp iac/backend.config "$$BACKUP_FILE"; \
			echo "$(GREEN)✅ Backed up existing config to $$BACKUP_FILE$(RESET)"; \
			echo "$(CYAN)💡 You can restore it anytime by copying it back to iac/backend.config$(RESET)"; \
		fi; \
	fi; \
	if [ "$$PROCEED" = true ]; then \
		command -v aws >/dev/null 2>&1 || { echo "$(RED)❌ AWS CLI not found. Install: https://aws.amazon.com/cli/$(RESET)"; exit 1; }; \
		aws sts get-caller-identity >/dev/null 2>&1 || { echo "$(RED)❌ AWS CLI not configured. Run: aws configure$(RESET)"; exit 1; }; \
		BUCKET_NAME=$${BUCKET_NAME:-}; \
		if [ -z "$$BUCKET_NAME" ]; then \
			echo -n "Enter a globally unique S3 bucket name for Terraform state: "; \
			read BUCKET_NAME; \
		fi; \
		if [ -z "$$BUCKET_NAME" ]; then \
			echo "$(RED)❌ Bucket name cannot be empty.$(RESET)"; \
			exit 1; \
		fi; \
		REGION=$${AWS_REGION:-ap-southeast-2}; \
		echo "$(BLUE)▶️ Creating S3 bucket '$$BUCKET_NAME' in region $$REGION...$(RESET)"; \
		if aws s3api head-bucket --bucket $$BUCKET_NAME --no-cli-pager 2>/dev/null; then \
			echo "$(YELLOW)⚠️  Bucket '$$BUCKET_NAME' already exists. Using existing bucket.$(RESET)"; \
		else \
			aws s3api create-bucket --bucket $$BUCKET_NAME --region $$REGION --create-bucket-configuration LocationConstraint=$$REGION --no-cli-pager > /dev/null; \
		fi; \
		echo "$(BLUE)▶️ Enabling versioning and encryption for '$$BUCKET_NAME'...$(RESET)"; \
		aws s3api put-bucket-versioning --bucket $$BUCKET_NAME --versioning-configuration Status=Enabled > /dev/null; \
		aws s3api put-bucket-encryption --bucket $$BUCKET_NAME --server-side-encryption-configuration '{"Rules": [{"ApplyServerSideEncryptionByDefault": {"SSEAlgorithm": "AES256"}}]}' > /dev/null; \
		echo "$(BLUE)▶️ Creating 'iac/backend.config' for local use...$(RESET)"; \
		ENVIRONMENT_NAME=$${ENVIRONMENT_NAME:-}; \
		if [ -z "$$ENVIRONMENT_NAME" ]; then \
			echo -n "Enter environment/branch name for Terraform state (optional, e.g., 'dev', 'feat-branch', or leave blank for default): "; \
			read ENVIRONMENT_NAME; \
		fi; \
		TF_STATE_KEY="aws-lambda-mcp/$${ENVIRONMENT_NAME}/terraform.tfstate"; \
		if [ -z "$$ENVIRONMENT_NAME" ]; then \
			TF_STATE_KEY="aws-lambda-mcp/terraform.tfstate"; \
		fi; \
		echo "bucket         = \"$$BUCKET_NAME\"" > iac/backend.config; \
		echo "key            = \"$$TF_STATE_KEY\"" >> iac/backend.config; \
		echo "region         = \"$$REGION\"" >> iac/backend.config; \
		echo "use_lockfile   = true" >> iac/backend.config; \
		echo "$(GREEN)✅ Backend setup complete!$(RESET)"; \
		echo "$(CYAN)ℹ️  Using native S3 state locking (Terraform 1.10+)$(RESET)"; \
		echo "Run '$(CYAN)make tf-init$(RESET)' to initialize Terraform with the new backend."; \
		{ grep -v '^TF_BACKEND_BUCKET=' .env 2>/dev/null; echo "TF_BACKEND_BUCKET=\"$$BUCKET_NAME\""; } > .env.tmp && mv .env.tmp .env 2>/dev/null || echo "TF_BACKEND_BUCKET=\"$$BUCKET_NAME\"" > .env; \
		echo "$(GREEN)✅ .env file updated with TF_BACKEND_BUCKET=$(RESET)"; \
	fi

login: ## 🔑 Authenticate AWS + Azure CLIs
	@echo "$(BLUE)🔐 Authenticating AWS + Azure CLIs...$(RESET)"
	@cd iac && $(MAKE) login

test-token: ## 🔑 Get OAuth token via device code flow + launch MCP Inspector (User Authentication)
	@echo "$(BLUE)🔑 Getting OAuth token via device code flow...$(RESET)"
	@lsof -ti:6274,6277 2>/dev/null | xargs kill -9 2>/dev/null || true
	@cd iac && $(MAKE) test-token

test-lambda: ## 🧪 Test Lambda directly (bypass Gateway)
	@echo "$(BLUE)🧪 Testing Lambda directly...$(RESET)"
	@cd iac && $(MAKE) test-lambda

logs: ## 📜 Tail Lambda logs
	@echo "$(BLUE)📜 Tailing Lambda logs (Ctrl+C to exit)...$(RESET)"
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

add-redirect-url: ## 🔗 Add custom OAuth redirect URL to the Entra ID application
	@echo "$(BLUE)🔗 Adding redirect URL to Entra ID application...$(RESET)"
	@cd iac && $(MAKE) add-redirect-url

remove-redirect-url: ## 🔗 Remove custom OAuth redirect URL from the Entra ID application
	@echo "$(BLUE)🔗 Removing redirect URL from Entra ID application...$(RESET)"
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

test-preview-inspector: deploy ## 🧪 Deploy and launch MCP Inspector with OAuth token for preview environment
	@echo "$(BLUE)🚀 Deploying and launching MCP Inspector for preview environment...$(RESET)"
	@cd iac && $(MAKE) test-token

.DEFAULT_GOAL := help