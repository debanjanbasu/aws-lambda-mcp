.PHONY: help schema build release test all deploy tf-init tf-plan tf-apply tf-destroy login test-token refresh test-lambda logs clean

help: ## Show this help
	@echo "Root Makefile - Lambda Build & Infrastructure"
	@echo ""
	@echo "Build Commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E 'schema|build|release|test' | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Infrastructure Commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E 'login|deploy|tf-|test-token|refresh|test-lambda|logs|clean' | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
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
	@cd iac && $(MAKE) test-token

refresh: ## Refresh expired access token
	@cd iac && $(MAKE) refresh

test-lambda: ## Test Lambda directly (bypass Gateway)
	@cd iac && $(MAKE) test-lambda

logs: ## Tail Lambda logs
	@cd iac && $(MAKE) logs

clean: ## Remove tokens and backups
	@cd iac && $(MAKE) clean

tf-destroy: ## Destroy Terraform resources
	@echo "Destroying Terraform resources..."
	@cd iac && terraform destroy -auto-approve

.DEFAULT_GOAL := help
