.PHONY: help schema build release test all

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

schema: ## Generate tool_schema.json
	@cargo run --bin generate-schema --features schema-gen

build: schema ## Build Lambda (debug)
	@cargo lambda build --bin aws-lambda-mcp

release: schema ## Build Lambda (release, ARM64)
	@cargo lambda build --release --arm64 --bin aws-lambda-mcp

test: ## Run tests
	@cargo test

all: test release ## Run tests and build release

.DEFAULT_GOAL := help
