# AI Assistant Instructions for AWS Lambda MCP

## Overview

This document provides guidelines for AI assistants working on the AWS Lambda MCP project, focusing on Rust code, build processes, and overall project management. For infrastructure-specific guidelines (Terraform, AWS resources), see `iac/AGENTS.md`. The project implements a secure, OAuth-authenticated bridge between Bedrock AI agents and custom tools using Rust, Terraform, and AWS infrastructure. Follow these rules to ensure code quality, security, and consistency.

## Table of Contents

- [Overview](#overview)
- [Project Overview](#project-overview)
- [Developer Quick Start](#developer-quick-start)
- [Commands](#commands)
- [Code Style Guidelines](#code-style-guidelines)
- [Troubleshooting](#troubleshooting)
- [Contributing as an AI Assistant](#contributing-as-an-ai-assistant)

## Project Overview

The AWS Lambda MCP is a Rust-based server implementing MCP for Bedrock AgentCore, enabling AI agents to discover and use external tools securely. Features a gateway interceptor for header propagation and token exchange. Key technologies: Rust (2024 edition), AWS Lambda (ARM64), Entra ID OAuth, CloudWatch logging, Terraform for infrastructure, CloudFormation for advanced gateway configuration.

**Developer Quick Start:**
- `make login` - Authenticate AWS and Azure CLIs
- `make deploy` - Build and deploy Lambdas to AWS (auto-installs tools if needed)
- `make test-token` - Get OAuth token and launch MCP Inspector for testing

## Gateway Interceptor

The project includes a gateway interceptor Lambda that sits between the AgentCore Gateway and the main MCP Lambda. The interceptor:

- Extracts authorization headers and custom headers from incoming requests
- Performs token exchange/validation (placeholder for actual implementation)
- Adds exchanged credentials and custom headers to the MCP request arguments
- Enables secure header propagation and token exchange workflows

The interceptor is deployed via CloudFormation to add interceptor configuration to the existing Terraform-managed gateway.

## Commands

### Build & Test
- `make build` - Debug build (main + interceptor Lambdas)
- `make release` - ARM64 production build with UPX compression (main + interceptor Lambdas)
- `make test` - Run all tests
- `cargo test <test_name>` - Run single test (e.g., `cargo test weather_integration`)
- `cargo clippy` - Run clippy with strict lints (denies unsafe code, unwrap, panic, etc.)
- `cargo fmt` - Format code
- `cargo fmt --check` - Validate code formatting
- `make schema` - Generate tool schemas (run after changing models/tools, then commit the updated tool_schema.json)
- `make check-tools` - Install/check required tools (Rust, Zig, cargo-lambda, UPX, jq, Terraform) *(Optional - called automatically by deploy)*
- `make help` - Show all available make commands
- `make all` - Run tests and build release

### Deployment
- `make deploy` - Build and deploy Lambdas to AWS (requires backend config)
- `make release` - Build optimized ARM64 Lambda binaries with UPX compression

### Infrastructure Setup
- `make setup-backend` - Create S3 backend for Terraform state with native locking
- `make check-backend-config` - Verify backend configuration exists

### Development Tools
- `make login` - Authenticate AWS and Azure CLIs
- `make test-token` - Get OAuth token and launch MCP Inspector
- `make test-lambda` - Test Lambda directly (bypass API Gateway)
- `make logs` - Tail Lambda CloudWatch logs
- `make clean` - Remove tokens and backups
- `make kill-inspector` - Kill running MCP Inspector processes
- `make oauth-config` - Display OAuth configuration
- `make add-redirect-url` - Add OAuth redirect URL to Entra ID app
- `make remove-redirect-url` - Remove OAuth redirect URL from Entra ID app
- `make update-secrets` - Update GitHub secrets from .env file
- `make update-deps` - Update all Rust and Terraform dependencies

### Terraform
- `make tf-init` - Initialize Terraform with backend config
- `make tf-plan` - Plan Terraform changes (builds Lambda first)
- `make tf-apply` - Apply Terraform changes (builds Lambda first)
- `make tf-destroy` - Destroy Terraform resources (generates schema first)

## Code Style Guidelines

- **Error Handling**: Return `Result<T>`, use `?` with `.context("Descriptive message")`, no `unwrap/expect/panic`. Example: `let data = fetch_data().context("Failed to fetch data")?;`
- **Imports**: Explicit imports only, no wildcards, clean up unused imports
- **Types**: Use explicit types for clarity, derive `Debug, Serialize, Deserialize, JsonSchema`. Example: `#[derive(Debug, Serialize, Deserialize, JsonSchema)] pub struct MyStruct { ... }`
- **Naming**: `snake_case` for variables/functions, `PascalCase` for types, `UPPERCASE` for constants
- **Functions**: Under 60 lines, `#[must_use]` on pure functions, document `# Errors`
- **Async**: Use `async/await` everywhere, no blocking I/O, `#[instrument]` for tracing. Use `tokio::spawn` for concurrency.
- **Security**: No unsafe code, no hardcoded secrets (use `std::env::var`), environment variables only
- **Performance**: Prefer `&str` over `String`, minimize allocations, `LazyLock` for globals
- **Makefiles**: Use `@echo "$(CYAN)Message$(RESET)"` for colored output in Makefiles (avoids shell escaping issues with printf)

## Troubleshooting

- **Build fails due to missing tools**: Run `make check-tools` to install dependencies.
- **Terraform destroy fails**: Ensure `tool_schema.json` exists; run `make schema` if needed.
- **OAuth issues**: Verify Entra ID app configuration with `make oauth-config`.
- **Linting errors**: Run `cargo clippy` and `cargo fmt` before committing.
- **Test failures**: Check logs with `make logs`; ensure all dependencies are installed.

## Contributing as an AI Assistant

- Prioritize security and code quality: Always run `make test` and `cargo clippy` after changes.
- Follow guidelines strictly: Reference this document for style and commands.
- Suggest improvements proactively: E.g., optimize async code or add error context.
- For complex changes: Propose plans and seek human review.
- Key files: See Cargo.toml for dependencies, iac/main.tf for infrastructure, src/ for code.
