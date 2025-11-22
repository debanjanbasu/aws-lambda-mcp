# Amazon Bedrock AgentCore Gateway (MCP Server)

[![Deploy](https://github.com/debanjanbasu/aws-lambda-mcp/actions/workflows/deploy.yml/badge.svg)](https://github.com/debanjanbasu/aws-lambda-mcp/actions/workflows/deploy.yml)
[![Security](https://github.com/debanjanbasu/aws-lambda-mcp/actions/workflows/checkov.yml/badge.svg)](https://github.com/debanjanbasu/aws-lambda-mcp/actions/workflows/checkov.yml)
[![CodeQL](https://github.com/debanjanbasu/aws-lambda-mcp/actions/workflows/codeql.yml/badge.svg)](https://github.com/debanjanbasu/aws-lambda-mcp/actions/workflows/codeql.yml)

Production-ready Model Context Protocol server implementation using Amazon Bedrock AgentCore Gateway. Secure, OAuth-authenticated bridge between Bedrock AI agents and custom tools.

## Table of Contents

- [Architecture](#architecture)
- [Features](#features)
- [One-Time Backend Setup](#one-time-backend-setup)
- [Quick Start](#quick-start)
- [Ephemeral Pull Request Environments](#ephemeral-pull-request-environments)
- [Automated Dependency Updates](#automated-dependency-updates)
- [Example: Weather Tool](#example-weather-tool)
- [Prerequisites](#prerequisites)
- [Initial Setup for GitHub Template Repositories](#initial-setup-for-github-template-repositories)
- [Structure](#structure)
- [Usage](#usage)
- [Troubleshooting](#troubleshooting)
- [Commands](#commands)
- [Schema Generation](#schema-generation)
- [Adding Tools](#adding-tools)
- [Configuration](#configuration)
- [Coding Standards](#coding-standards)
- [Contributing](#contributing)

## Architecture

```
Client → Entra ID (PKCE) → AgentCore Gateway → Lambda (Rust) → External APIs
                                    ↓
                            JWT Validation (OIDC)
```

**Stack**: ARM64 Lambda (~1.3MB UPX) | Entra ID OAuth | CloudWatch (90d retention)

**License**: MIT

## Model Context Protocol Implementation

This is a **Model Context Protocol (MCP) server** implemented as an AWS Lambda function for Amazon Bedrock AgentCore. MCP is an open-source specification that enables AI agents to discover and interact with external tools and APIs in a standardized way. This server uses the `rmcp` crate's `#[tool]` macro for MCP-compliant schema generation.

The Bedrock AgentCore Gateway is configured with a `SEMANTIC` search type, which enables intelligent tool selection. This means it can understand natural language queries, match tool descriptions and parameters, and provide context-aware tool recommendations, significantly improving the agent's ability to utilize available tools effectively.

## Features

- **ARM64/Graviton** - 20% cheaper, UPX compressed to 1.3MB
- **Secretless OAuth** - PKCE flow, no client secrets
- **JWT Validation** - OIDC discovery per request
- **Zero Unsafe** - No `unwrap/expect/panic/unsafe`, strict lints
- **Structured Tracing** - JSON logs for CloudWatch
- **Dead Letter Queue** - Failed invocations stored in SQS for debugging
- **Auto Schemas** - Generated from code annotations
- **Fast Cold Start** - Minimal deps, optimized binary
- **Free Tier** - Typical usage $0/month

## One-Time Backend Setup

Before you can deploy, you need to run a one-time setup command to create the Terraform backend infrastructure:

```bash
make setup-backend
```

This command will:
1. Prompt you for a unique S3 bucket name
2. Create the S3 bucket for Terraform state storage
3. Enable versioning and encryption on the bucket
4. Configure native S3 state locking (Terraform 1.10+)
5. Generate the `iac/backend.config` file

After setup, you can deploy your infrastructure with:
```bash
make deploy
```

**Important**: The `backend.config` file is essential for all Terraform operations. The Makefiles now include smart backend checking that will guide you if this file is missing.

## Quick Start

```bash
make setup-backend # One-time backend setup (S3 with native locking)
make deploy        # Build and deploy to AWS
make test-token    # Get OAuth token + launch MCP Inspector
```

The `test-token` command automatically copies the token to clipboard (macOS/Linux/WSL) and provides instructions for testing with the MCP Inspector.

## Ephemeral Pull Request Environments

This repository automatically creates isolated test environments for each pull request:

- 🌱 **Automatic Deployment**: When you open a non-draft PR, an ephemeral environment is automatically created
- 🔗 **Isolated Testing**: Each PR gets its own Gateway URL and backend resources
- 🧪 **Easy Testing**: Use the same `make test-token` workflow to test your changes
- 🗑️ **Automatic Cleanup**: Environments are destroyed when the PR is closed or merged

### Manual Environment Management

To manually trigger an environment deployment or destruction:
```bash
# Deploy a manual environment (replace 123 with your PR number)
gh workflow run preview-environment.yml -f action=deploy -f pr_number=123

# Destroy a manual environment
gh workflow run preview-environment.yml -f action=destroy -f pr_number=123
```

**Note**: When running manually, the `pr_number` input is required to namespace the environment resources (e.g., `preview-123`). Use the actual PR number if you are debugging a specific PR, or any unique number for a scratch environment.

## Automated Dependency Updates

Dependabot automatically creates PRs for:
- 🦀 **Rust dependencies** - Cargo.toml updates
- 🏗️ **Terraform providers** - AWS, Entra ID, and other providers
- ⚙️ **GitHub Actions** - Workflow action updates

Updates are automatically tested and merged when all checks pass.

## Example: Weather Tool

Included working tool demonstrates the pattern:
- Geocodes locations (Open-Meteo API)
- Fetches current weather
- Returns temperature in local units (°C/°F by country)
- WMO weather code + wind speed

## Prerequisites

- **Rust** (edition 2024)
- **cargo-lambda**: `cargo install cargo-lambda`
- **UPX**: `brew install upx` (macOS) | `apt install upx-ucl` (Linux)
- **Zig**: `brew install zig` (macOS) | `apt install zig` (Linux)
- **jq**: `brew install jq` (macOS) | `apt install jq` (Linux)
- **Terraform** (latest)
- **AWS CLI** (configured)
- **Azure CLI** (configured)

**Note**: Running `make release` will automatically install missing tools locally.

## Initial Setup for GitHub Template Repositories

When using this repository as a GitHub template, you'll need to set up several secrets in your repository settings for the GitHub Actions workflows to function properly.

**Resource Naming**: The system automatically generates unique resource names by appending a random suffix (e.g., `aws-agentcore-gateway-a1b2c3`) to prevent conflicts when multiple deployments exist in the same AWS account.

### Required GitHub Secrets

| Secret Name | Description | Setup Instructions |
|-------------|-------------|--------------------|
| `AWS_IAM_ROLE_ARN` | AWS IAM Role ARN for GitHub Actions OIDC authentication | [AWS GitHub Actions Setup](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/configuring-openid-connect-in-amazon-web-services) |
| `AZURE_CLIENT_ID` | Entra ID App Registration Client ID | [Azure GitHub Actions Setup](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/configuring-openid-connect-in-azure) |
| `AZURE_TENANT_ID` | Entra ID Tenant ID | [Azure GitHub Actions Setup](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/configuring-openid-connect-in-azure) |
| `TF_BACKEND_BUCKET` | S3 Bucket name for Terraform state storage | Run `make setup-backend` after setting AWS credentials |
| `APP_PRIVATE_KEY` | PEM private key for the GitHub App `@brown-ninja-bot` (multi-line). Used to mint short-lived installation tokens for CI automation. | Create a GitHub App (Settings → Developer settings → GitHub Apps), generate and download the private key, then add the PEM contents as the secret `APP_PRIVATE_KEY` in this repository's Settings → Secrets & variables → Actions. |
| `APP_ID` | Numeric GitHub App ID for `@brown-ninja-bot`. Used together with the private key to mint JWTs. | Add the numeric App ID as the secret `APP_ID` in repository secrets. |


To validate your GitHub App setup you can use the provided test workflow:

```bash
# Trigger the test workflow which mints an installation token and validates it
# from the Actions tab: "Test: GitHub App Installation Token" → Run workflow
# or via CLI:
# gh workflow run test-github-app-token.yml
```



### Optional GitHub Secrets

| Secret Name | Description | Default |
|-------------|-------------|---------|
| `PROJECT_NAME_SUFFIX` | Custom suffix for resource names (e.g., "prod", "dev"). If not set, a random suffix is auto-generated | Random 6-char string |

### Setting Up AWS Authentication

1. Follow [GitHub's documentation](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/configuring-openid-connect-in-amazon-web-services) to configure OIDC between GitHub and AWS
2. Create an IAM role with the necessary permissions for Lambda, API Gateway, and S3
3. Set the `AWS_IAM_ROLE_ARN` secret to the ARN of this role

### Setting Up Entra ID Authentication

1. Follow [GitHub's documentation](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/configuring-openid-connect-in-azure) to configure OIDC between GitHub and Azure
2. Register a GitHub Actions application in Entra ID
3. Set the `AZURE_CLIENT_ID` and `AZURE_TENANT_ID` secrets

### Setting Up Terraform Backend

After configuring AWS authentication:
1. Run `make setup-backend` locally to create the S3 bucket. This command will also automatically add the `TF_BACKEND_BUCKET` value to your local `.env` file.
2. Use `make update-secrets` to push these values to your GitHub repository secrets.



### Updating GitHub Secrets

To update your GitHub repository secrets for **both GitHub Actions and Dependabot**, create a `.env` file in the root of the project with the secrets you wish to update (e.g., `MY_SECRET="myvalue"`). You can use the provided `.env.example` file as a template for the required and optional secrets.

Then, run the following command:

```bash
make update-secrets
```

This command will read the `.env` file and use the `gh CLI` to set or update the corresponding repository secrets for both GitHub Actions and Dependabot.

**Important**: Ensure your `.env` file is in your `.gitignore` to prevent accidentally committing sensitive information.

### Using with opencode.ai

This repository is pre-configured to work with [opencode.ai](https://opencode.ai), an AI-powered development assistant that can help you build, debug, and maintain your MCP server. The project includes:

- Pre-configured GitHub Actions workflows that integrate with opencode.ai
- Automatic schema generation for tool discovery
- Standardized MCP implementation patterns
- Built-in testing and debugging tools

To use opencode.ai with this project:

1. Visit [opencode.ai](https://opencode.ai) and sign up for an account
2. Install the opencode CLI: `npm install -g opencode`
3. Authenticate: `opencode login`
4. Navigate to your project directory and run: `opencode`

The opencode assistant will automatically detect your project structure and provide context-aware help for:
- Adding new tools and capabilities
- Debugging deployment issues
- Optimizing performance
- Following MCP best practices
- Integrating with other AI services

For more information, see the [opencode.ai GitHub documentation](https://opencode.ai/docs/github/).

## Structure

```
src/
├── main.rs              # Lambda bootstrap + tracing
├── handler.rs           # Event handler
├── models/              # Request/response types (JsonSchema)
│   └── weather.rs
├── tools/               # Tool implementations (#[tool] macro)
│   └── weather.rs
├── http/                # Global HTTP_CLIENT
└── bin/
    └── generate_schema.rs

macros/                  # Custom #[tool] proc macro
└── src/lib.rs
```

## Usage

### Build & Test
```bash
make schema    # Generate tool_schema.json
make build     # Debug build
make release   # ARM64 + UPX (~1.3MB)
make test      # Run tests
make all       # Test + release build
```

### Deploy
```bash
make setup-backend # One-time backend setup
make deploy        # Build and deploy to AWS
make tf-destroy    # Destroy infrastructure
```

### Development
```bash
make test-token   # OAuth + Inspector (token auto-copied)
make test-lambda  # Direct Lambda test
make logs         # Tail CloudWatch logs
make login        # AWS + Azure auth
make clean        # Remove tokens/backups
```

### Advanced Terraform Operations
```bash
make tf-init   # Initialize Terraform
make tf-plan   # Plan changes
make tf-apply  # Apply changes
```

For full infrastructure commands: `cd iac && make help`

## Troubleshooting

### Gateway Exception Logging

Control Gateway exception logging verbosity in `iac/terraform.tfvars`:

```hcl
# Disabled (default) - Minimal error information for security
gateway_exception_level = null

# Error level - Only error messages
gateway_exception_level = "ERROR"

# Warning level - Warning and error messages
gateway_exception_level = "WARN"

# Info level - Informational, warning, and error messages
gateway_exception_level = "INFO"

# Debug level - Most verbose logging (use only for troubleshooting)
gateway_exception_level = "DEBUG"
```

Redeploy: `cd iac && terraform apply -auto-approve`

⚠️ Security considerations:
Higher verbosity levels may expose sensitive information in error responses. 
Use DEBUG/INFO only for troubleshooting, not in production environments.
Disable (set to `null`) after troubleshooting to avoid exposing sensitive data.

### Lambda Debug Logs

Edit `iac/variables.tf`:
```hcl
variable "rust_log_level" {
  default = "debug"  # or "trace"
}
```

Redeploy and view: `make logs`

**Production**: Set to `"info"` to avoid logging sensitive payloads

### Common Issues

| Issue | Solution |
|-------|----------|
| "Access denied" | Gateway IAM needs both `bedrock.amazonaws.com` AND `bedrock-agentcore.amazonaws.com` principals |
| "Invalid Bearer token" | Token needs `api://CLIENT_ID/access_as_user` scope. Run `make test-token` |
| Lambda timeout | Increase `lambda_timeout` in `iac/variables.tf` |

## Commands

### Main Commands
| Command | Description |
|---------|-------------|
| `make help` | Show all commands with colored output |
| `make schema` | Generate tool_schema.json |
| `make build` | Debug build |
| `make release` | ARM64 + UPX production build |
| `make test` | Run tests |
| `make all` | Test + release build |
| `make deploy` | Build and deploy to AWS (smart backend checking) |
| `make setup-backend` | One-time backend setup |
| `make test-token` | OAuth + Inspector (clipboard) |
| `make test-lambda` | Direct Lambda test |
| `make logs` | Tail CloudWatch logs |
| `make update-deps` | Update all dependencies |

### Infrastructure Commands
| Command | Description |
|---------|-------------|
| `make login` | AWS + Azure auth |
| `make tf-init` | Initialize Terraform (smart backend checking) |
| `make tf-plan` | Plan Terraform changes |
| `make tf-apply` | Apply Terraform changes |
| `make tf-destroy` | Destroy infrastructure |
| `make clean` | Remove tokens/backups |
| `make oauth-config` | Display OAuth configuration details |
| `make add-redirect-url` | Add custom OAuth redirect URL to terraform.tfvars |

For advanced infrastructure commands: `cd iac && make help`

## Schema Generation

Generates Amazon Bedrock AgentCore schemas from code using custom `#[tool]` macro (not MCP):

```rust
use crate::macros::tool;

#[tool(description = "Get current weather for a location")]
#[instrument(fields(location = %request.location))]
pub async fn get_weather(request: WeatherRequest) -> Result<WeatherResponse> {
    // implementation
}
```

Run `make schema` → generates `tool_schema.json` with:
- Tool name from function name
- Description from macro attribute
- Input/output schemas from types (via `schemars`)
- Bedrock-compatible format (no enums, inlined types)

## Adding Tools

**1. Model** (`src/models/your_tool.rs`):
```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct YourRequest {
    #[schemars(description = "Input description")]
    pub input: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct YourResponse {
    pub result: String,
}
```

**2. Tool** (`src/tools/your_tool.rs`):
```rust
#[tool(description = "Clear, detailed description")]
#[instrument(fields(input = %request.input))]
pub async fn your_tool(request: YourRequest) -> Result<YourResponse> {
    // implementation
}
```

**3. Register** in `src/bin/generate_schema.rs`:
```rust
tool_entry!(your_tool::YOUR_TOOL_METADATA, YourRequest, YourResponse),
```

**4. Generate**: `make schema`

**5. Route**: Update `handler.rs` to call your tool

## Configuration

**Lambda** (`Cargo.toml`):
```toml
[package.metadata.lambda.deploy]
memory = 128
timeout = 30
tracing = "active"
```

**Infrastructure**: Edit `iac/terraform.tfvars` for custom settings

## Coding Standards

See [AGENTS.md](./AGENTS.md) for full guidelines.

**Rules**:
- ✅ `Result<T>` + `?` with `.context()`
- ✅ `#[instrument]` for tracing
- ✅ `#[must_use]` on pure functions
- ❌ No `unwrap/expect/panic/unsafe`
- ❌ No blocking I/O in async
- ❌ No wildcard imports

**Dependencies**: `lambda_runtime` | `tokio` | `serde` | `schemars` | `reqwest` | `tracing` | `anyhow`

## Contributing

1. Read [AGENTS.md](./AGENTS.md)
2. `cargo clippy -- -D warnings`
3. `make schema` if models changed
4. `make release` succeeds
