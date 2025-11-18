# AI Assistant Instructions for AWS Lambda MCP

**Developer Quick Start:**
- `make login` - Authenticate AWS and Azure CLIs
- `make deploy` - Build and deploy Lambda to AWS (auto-installs tools if needed)
- `make test-token` - Get OAuth token and launch MCP Inspector for testing

**Build/Lint/Test Commands:**
- `make build` - Debug build
- `make release` - ARM64 production build with UPX compression
- `make test` - Run all tests
- `cargo test <test_name>` - Run single test
- `cargo clippy` - Run clippy with strict lints (denies unsafe code, unwrap, panic, etc.)
- `cargo fmt` - Format code
- `make schema` - Generate tool schemas
- `make check-tools` - Install/check required tools (Rust, Zig, cargo-lambda, UPX, jq, Terraform) *(Optional - called automatically by deploy)*
- `make help` - Show all available make commands

**Code Style Guidelines:**
- **Error Handling**: Return `Result<T>`, use `?` with `.context()`, no `unwrap/expect/panic`
- **Imports**: Explicit imports only, no wildcards, clean up unused imports
- **Types**: Use explicit types for clarity, derive `Debug, Serialize, Deserialize, JsonSchema`
- **Naming**: `snake_case` for variables/functions, `PascalCase` for types, `UPPERCASE` for constants
- **Functions**: Under 60 lines, `#[must_use]` on pure functions, document `# Errors`
- **Async**: Use `async/await` everywhere, no blocking I/O, `#[instrument]` for tracing
- **Security**: No unsafe code, no hardcoded secrets, environment variables only
- **Performance**: Prefer `&str` over `String`, minimize allocations, `LazyLock` for globals
- **Makefiles**: Use `@echo "$(CYAN)Message$(RESET)"` for colored output in Makefiles (avoids shell escaping issues with printf)

**Deployment Commands:**
- `make deploy` - Build and deploy Lambda to AWS (requires backend config)
- `make release` - Build optimized ARM64 Lambda binary with UPX compression

**Infrastructure Setup:**
- `make setup-backend` - Create S3 backend for Terraform state with native locking
- `make check-backend-config` - Verify backend configuration exists

**Development Tools:**
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

**Terraform Commands:**
- `make tf-init` - Initialize Terraform with backend config
- `make tf-plan` - Plan Terraform changes (builds Lambda first)
- `make tf-apply` - Apply Terraform changes (builds Lambda first)
- `make tf-destroy` - Destroy Terraform resources
