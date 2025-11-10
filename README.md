# AWS Bedrock AgentCore Gateway

Production-ready AWS Lambda function in Rust for AWS Bedrock AgentCore tool execution. Secure, OAuth-authenticated bridge between Bedrock AI agents and custom tools.

## Architecture

```
Client → Entra ID (PKCE) → AgentCore Gateway → Lambda (Rust) → External APIs
                                    ↓
                            JWT Validation (OIDC)
```

**Stack**: ARM64 Lambda (~1.3MB UPX) | Entra ID OAuth | CloudWatch (3d retention)

## ⚠️ Not an MCP Server

This is an **AWS Lambda function** for AWS Bedrock AgentCore, not a Model Context Protocol server. Uses custom `#[tool]` macro for Bedrock-specific schemas.

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

This project uses an S3 bucket to store Terraform's remote state securely. Before you can deploy, you need to run a one-time setup command.

1.  **Create the Backend Infrastructure:**
    ```bash
    make setup-backend
    ```
    This command will prompt you for a unique S3 bucket name, then use the AWS CLI to create the S3 bucket and a DynamoDB table for state locking. It also creates a local `iac/backend.config` file, which is ignored by Git.

2.  **Create GitHub Secrets:**
    For the GitHub Actions workflow to use the remote backend, you must add the following secrets in your repository settings under **Settings > Secrets and variables > Actions**:
    *   `TF_BACKEND_BUCKET`: The name of the S3 bucket you just created.
    *   `TF_BACKEND_DYNAMODB_TABLE`: The name of the DynamoDB table (`terraform-state-lock-mcp`).

After this one-time setup, you can proceed with the normal deployment workflow.

## Quick Start

```bash
make login        # AWS + Azure auth
make deploy       # Build + deploy
make test-token   # Get OAuth token (auto-copied to clipboard)
```

Token is automatically copied to clipboard (macOS/Linux/WSL). Paste into MCP Inspector when prompted.

## Ephemeral Pull Request Environments

This repository automatically creates isolated test environments for each pull request:

- 🌱 **Automatic Deployment**: When you open a non-draft PR, an ephemeral environment is automatically created
- 🔗 **Isolated Testing**: Each PR gets its own Gateway URL and backend resources
- 🧪 **Easy Testing**: Use the same `make test-token` workflow to test your changes
- 🗑️ **Automatic Cleanup**: Environments are destroyed when the PR is closed or merged

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
- 🏗️ **Terraform providers** - AWS, Azure AD, and other providers
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

### Build
```bash
make schema    # Generate tool_schema.json
make build     # Debug build
make release   # ARM64 + UPX (~1.3MB)
make test      # Run tests
```

### Deploy
```bash
make login     # AWS + Azure auth
make deploy    # Build + Terraform apply
```

### Test
```bash
make test-token   # OAuth + Inspector (token auto-copied)
make refresh      # Refresh expired token
make test-lambda  # Direct Lambda test
make logs         # Tail CloudWatch logs
```

## Troubleshooting

### Gateway Debug Mode

Enable detailed errors in `iac/terraform.tfvars`:
```hcl
gateway_enable_debug = true  # Shows _meta.debug in responses
```

Redeploy: `cd iac && terraform apply -auto-approve`

⚠️ Disable after troubleshooting (may expose sensitive data)

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

| Command | Description |
|---------|-------------|
| `make help` | Show all commands |
| `make schema` | Generate tool_schema.json |
| `make build` | Debug build |
| `make release` | ARM64 + UPX production build |
| `make test` | Run tests |
| `make all` | Test + release build |
| `make login` | AWS + Azure auth |
| `make deploy` | Build + Terraform apply |
| `make test-token` | OAuth + Inspector (clipboard) |
| `make refresh` | Refresh expired token |
| `make test-lambda` | Direct Lambda test |
| `make logs` | Tail CloudWatch logs |
| `make clean` | Remove tokens/backups |
| `make tf-destroy` | Destroy infrastructure |

## Schema Generation

Generates AWS Bedrock AgentCore schemas from code using custom `#[tool]` macro (not MCP):

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