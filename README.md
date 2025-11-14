# Amazon Bedrock AgentCore Gateway (MCP Server)

Production-ready Model Context Protocol server implementation using Amazon Bedrock AgentCore Gateway. Secure, OAuth-authenticated bridge between Bedrock AI agents and custom tools.

## Architecture

```
Client → Entra ID (PKCE) → AgentCore Gateway → Lambda (Rust) → External APIs
                                    ↓
                            JWT Validation (OIDC)
```

**Stack**: ARM64 Lambda (~1.3MB UPX) | Entra ID OAuth | CloudWatch (3d retention)

**License**: MIT

## Model Context Protocol Implementation

This is a **Model Context Protocol server** implemented as an AWS Lambda function for Amazon Bedrock AgentCore. Uses the `rmcp` crate's `#[tool]` macro for MCP-compliant schema generation.

## Features

- **ARM64/Graviton** - 20% cheaper, UPX compressed to 1.3MB
- **Secretless OAuth** - PKCE flow, no client secrets
- **JWT Validation** - OIDC discovery per request
- **Zero Unsafe** - No `unwrap/expect/panic/unsafe`, strict lints
- **Structured Tracing** - JSON logs for CloudWatch
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
4. Create a DynamoDB table for state locking
5. Generate the `iac/backend.config` file

After setup, you can deploy your infrastructure with:
```bash
make deploy
```

**Important**: The `backend.config` file is essential for all Terraform operations. The Makefiles now include smart backend checking that will guide you if this file is missing.

## Quick Start

```bash
make setup-backend # One-time backend setup (S3 + DynamoDB)
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
# Deploy a manual environment
gh workflow run pr-environment.yml -f action=deploy

# Destroy a manual environment
gh workflow run pr-environment.yml -f action=destroy
```

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
- **AWS CLI** (configured)
- **Azure CLI** (configured)

## Initial Setup for GitHub Template Repositories

When using this repository as a GitHub template, you'll need to set up several secrets in your repository settings for the GitHub Actions workflows to function properly.

### Required GitHub Secrets

| Secret Name | Description | Setup Instructions |
|-------------|-------------|--------------------|
| `AWS_IAM_ROLE_ARN` | AWS IAM Role ARN for GitHub Actions OIDC authentication | [AWS GitHub Actions Setup](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/configuring-openid-connect-in-amazon-web-services) |
| `AZURE_CLIENT_ID` | Entra ID App Registration Client ID | [Azure GitHub Actions Setup](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/configuring-openid-connect-in-azure) |
| `AZURE_TENANT_ID` | Entra ID Tenant ID | [Azure GitHub Actions Setup](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/configuring-openid-connect-in-azure) |
| `TF_BACKEND_BUCKET` | S3 Bucket name for Terraform state storage | Run `make setup-backend` after setting AWS credentials |
| `TF_BACKEND_DYNAMODB_TABLE` | DynamoDB Table name for Terraform state locking | Run `make setup-backend` after setting AWS credentials |

### Optional GitHub Secrets (for Gemini workflows)

| Secret Name | Description | Setup Instructions |
|-------------|-------------|--------------------|
| `GEMINI_API_KEY` | Google Gemini API Key for AI-powered workflows | [Google AI Studio](https://aistudio.google.com/) |
| `GOOGLE_API_KEY` | Google API Key for additional Google services | [Google Cloud Console](https://console.cloud.google.com/apis/credentials) |
| `APP_PRIVATE_KEY` | GitHub App Private Key for advanced workflows | [GitHub App Setup](https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/managing-private-keys-for-github-apps) |

### Setting Up AWS Authentication

1. Follow [GitHub's documentation](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/configuring-openid-connect-in-amazon-web-services) to configure OIDC between GitHub and AWS
2. Create an IAM role with the necessary permissions for Lambda, API Gateway, S3, and DynamoDB
3. Set the `AWS_IAM_ROLE_ARN` secret to the ARN of this role

### Setting Up Entra ID Authentication

1. Follow [GitHub's documentation](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/configuring-openid-connect-in-azure) to configure OIDC between GitHub and Azure
2. Register a GitHub Actions application in Entra ID
3. Set the `AZURE_CLIENT_ID` and `AZURE_TENANT_ID` secrets

### Setting Up Terraform Backend

After configuring AWS authentication:
1. Run `make setup-backend` locally to create the S3 bucket and DynamoDB table. This command will also automatically add the `TF_BACKEND_BUCKET` and `TF_BACKEND_DYNAMODB_TABLE` values to your local `.env` file.
2. Use `make update-secrets` to push these values to your GitHub repository secrets.

### Updating GitHub Secrets

To update your GitHub repository secrets for **both GitHub Actions and Dependabot**, create a `.env` file in the root of the project with the secrets you wish to update (e.g., `MY_SECRET="myvalue"`). You can use the provided `.env.example` file as a template for the required and optional secrets.

Then, run the following command:

```bash
make update-secrets
```

This command will read the `.env` file and use the `gh CLI` to set or update the corresponding repository secrets for both GitHub Actions and Dependabot.

**Important**: Ensure your `.env` file is in your `.gitignore` to prevent accidentally committing sensitive information.

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