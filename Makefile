.PHONY: help schema build release test all deploy tf-init tf-plan tf-apply tf-destroy

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

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

tf-init: ## Initialize Terraform
	@echo "Initializing Terraform..."
	@cd iac && terraform init

tf-plan: release ## Plan Terraform changes (builds Lambda first)
	@echo "Planning Terraform deployment..."
	@cd iac && terraform plan

tf-apply: release ## Apply Terraform changes (builds Lambda first)
	@echo "Applying Terraform deployment..."
	@cd iac && terraform apply

tf-destroy: ## Destroy Terraform resources
	@echo "Destroying Terraform resources..."
	@cd iac && terraform destroy

deploy: tf-apply ## Build and deploy to AWS (alias for tf-apply)

.DEFAULT_GOAL := help
