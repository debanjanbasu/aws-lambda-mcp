# AWS Lambda Bedrock Agent Gateway

A production-ready AWS Lambda function that serves as a gateway for Amazon Bedrock Agents, written in Rust. This provides a secure, high-performance bridge between Bedrock AI agents and custom tool implementations with OAuth authentication via Entra ID.

## Architecture

```
MCP Client → Entra ID OAuth (PKCE) → Bedrock Gateway → Lambda (Rust) → External APIs
                                            ↓
                                   JWT Validation (OIDC)
```

**Components**:
- **AWS Lambda**: ARM64 Rust binary (~1.3MB with UPX compression)
- **Bedrock Agent Core Gateway**: MCP protocol endpoint
- **Entra ID OAuth**: Secretless PKCE flow
- **CloudWatch Logs**: 3-day retention for cost optimization

## Important: Not an MCP Server

**This is NOT a Model Context Protocol (MCP) server.** It's an AWS Lambda function specifically designed for Amazon Bedrock Agent integration. We use a custom `#[tool]` macro approach tailored for AWS Bedrock's schema requirements.

## Features

- **Rust Performance**: Compiled for ARM64/Graviton (20% cheaper, UPX compressed to 1.3MB)
- **Secretless OAuth**: PKCE flow with Entra ID (no client secrets)
- **JWT Validation**: Every request validated via OIDC discovery
- **Security First**: No `unsafe` code, no `unwrap/expect/panic`, strict clippy lints
- **Observability**: Structured tracing with JSON output for CloudWatch
- **Auto-Generated Schemas**: Tool schemas from code annotations
- **Low Cold Start**: Minimal dependencies and optimized binary size
- **Cost Optimized**: Free tier covers typical usage ($0/month)

## Quick Start

```bash
# 1. Authenticate to AWS and Azure
make login

# 2. Deploy infrastructure
make deploy

# 3. Test with OAuth token
make test-token
```

This authenticates, deploys everything, and launches the MCP Inspector automatically.

## Example Tool: Weather Service

The project includes a working weather tool that demonstrates the complete pattern:
- Geocodes locations using Open-Meteo API
- Fetches current weather data
- Returns temperature in local units (Celsius/Fahrenheit by country)
- Includes WMO weather code and wind speed

## Prerequisites

- Rust toolchain (edition 2024)
- [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda) - `cargo install cargo-lambda`
- [UPX](https://upx.github.io/) - For binary compression
  - macOS: `brew install upx`
  - Linux: `apt-get install upx-ucl`
- AWS CLI configured
- Azure CLI configured

## Project Structure

```
src/
├── main.rs                   # Lambda bootstrap & tracing initialization
├── handler.rs                # Lambda event handler
├── lib.rs                    # Library exports for schema generation
├── macros/                   # Re-export of tool attribute macro
│   └── mod.rs
├── models/                   # Domain models with JsonSchema
│   ├── mod.rs
│   └── weather.rs           # Weather request/response types
├── tools/                    # Tool implementations (use #[tool] macro)
│   ├── mod.rs
│   └── weather.rs           # Weather API integration
├── http/                     # HTTP client configuration
│   ├── mod.rs
│   └── client.rs            # Global HTTP_CLIENT with tracing
└── bin/
    └── generate_schema.rs   # Schema generator binary

macros/                       # Proc macro crate
├── Cargo.toml
└── src/
    └── lib.rs               # #[tool] attribute macro implementation
```

## Getting Started

### 1. Clone and Build

```bash
git clone <your-repo-url>
cd aws-lambda-mcp

# Generate tool schema
make schema

# Build for development
make build

# Build for production (ARM64/Graviton with UPX compression)
make release
```

### 2. Deploy to AWS

```bash
# Login to AWS and Azure
make login

# Deploy everything (builds Lambda + applies Terraform)
make deploy
```

### 3. Test the Gateway

```bash
# Get OAuth token and launch MCP Inspector
make test-token

# Refresh expired token
make refresh

# Test Lambda directly (bypass Gateway)
make test-lambda

# View logs
make logs
```

## Troubleshooting

### Enable Detailed Error Logging

If you're getting "An internal error occurred" from the Gateway, enable debug logging:

1. Edit `iac/terraform.tfvars` (or create it):
```hcl
gateway_enable_debug = true
```

2. Redeploy:
```bash
cd iac && terraform apply -auto-approve
```

3. Test again - errors will now include detailed context with `_meta.debug` fields

4. **Important**: Disable debug after troubleshooting:
```hcl
gateway_enable_debug = false  # Default
```

Debug mode provides detailed error information but may expose sensitive details in error responses.

### Common Issues

**"Access denied while invoking Lambda"**: Gateway IAM role needs both `bedrock.amazonaws.com` AND `bedrock-agentcore.amazonaws.com` service principals. See `iac/main.tf` trust policy comments.

**"Invalid Bearer token"**: Token requires `api://CLIENT_ID/access_as_user` scope. Run `make test-token` to get a fresh token.

**Lambda timeout**: Increase `lambda_timeout` in `variables.tf` if your tools need more time.

### Debug Logging and Security

The Lambda function conditionally logs event payloads based on the `RUST_LOG` environment variable:

- **Production (`RUST_LOG=info/warn/error`)**: Only event size is logged. Event payload is NOT included in logs for security.
- **Debug/Troubleshooting (`RUST_LOG=debug/trace`)**: Full event payload and context are logged to CloudWatch for debugging.

To enable detailed logging for troubleshooting:

1. Edit `iac/variables.tf`:
```hcl
variable "rust_log_level" {
  default     = "debug"  # or "trace" for even more detail
}
```

2. Redeploy:
```bash
cd iac && terraform apply -auto-approve
```

3. View logs with event details:
```bash
make logs
```

4. **Important**: Set back to `info` for production to avoid logging sensitive data:
```hcl
variable "rust_log_level" {
  default     = "info"
}
```

The tracing configuration automatically:
- Enables `with_current_span(true)` for debug/trace levels (shows field values)
- Uses `skip_if` in `#[instrument]` to exclude event from spans when not debugging
- Keeps CloudWatch logs lean and secure in production

## Available Commands

All commands can be run from the root directory:

```bash
# Build Commands
make help         # Show all commands
make schema       # Generate tool_schema.json
make build        # Build Lambda (debug)
make release      # Build Lambda (ARM64, UPX compressed)
make test         # Run tests
make all          # Run tests + build release

# Infrastructure Commands
make login        # Authenticate AWS + Azure CLIs
make deploy       # Build + deploy to AWS
make test-token   # OAuth flow + launch Inspector
make refresh      # Refresh expired access token
make test-lambda  # Test Lambda directly
make logs         # Tail Lambda logs
make clean        # Remove tokens and backups
make tf-destroy   # Destroy infrastructure
```

## Tool Schema Generation

The project automatically generates AWS Bedrock-compatible tool schemas from your Rust code using a **custom** `#[tool]` attribute macro (not the rmcp SDK):

1. Define your models in `src/models/` with `#[derive(JsonSchema)]`
2. Implement your tool function in `src/tools/` with `#[tool(description = "...")]`
3. Run `make schema` to generate `tool_schema.json`

The schema generator:
- Reads metadata exported by the custom `#[tool]` macro
- Extracts function names (becomes tool name)
- Uses macro descriptions (becomes tool description)
- Generates JSON schemas from your models with `schemars`
- Produces AWS Bedrock-compatible output (no enums, inlined types)

**Note**: We use a custom macro specifically designed for Bedrock Agent schemas. This is different from MCP tool schemas. See [COPILOT.md](./COPILOT.md) for architectural decisions.

Example:
```rust
use crate::macros::tool;  // Our custom macro, not rmcp
use crate::models::{WeatherRequest, WeatherResponse};

#[tool(description = "Get current weather for a location")]
#[instrument(fields(location = %request.location))]
pub async fn get_weather(request: WeatherRequest) -> Result<WeatherResponse> {
    // implementation
}
```

This generates:
- Function metadata constant: `GET_WEATHER_METADATA`
- Tool schema entry in `tool_schema.json` with name "get_weather"
- Input/output schemas derived from `WeatherRequest`/`WeatherResponse` that conform to Bedrock requirements

## Adding New Tools

1. Create model types in `src/models/your_tool.rs`:
```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct YourRequest {
    /// Input parameter description
    pub input: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct YourResponse {
    /// Output parameter description
    pub result: String,
}
```

2. Implement the tool in `src/tools/your_tool.rs`:
```rust
use crate::macros::tool;  // Our custom macro
use crate::models::{YourRequest, YourResponse};
use anyhow::Result;
use tracing::instrument;

/// # Errors
/// Returns error if something fails.
#[tool(description = "Your tool description here - be detailed and clear")]
#[instrument(fields(input = %request.input))]
pub async fn your_tool_name(request: YourRequest) -> Result<YourResponse> {
    // Implementation
    Ok(YourResponse { result: String::new() })
}
```

3. Export in respective `mod.rs` files:
```rust
// src/models/mod.rs
pub mod your_tool;
pub use your_tool::*;

// src/tools/mod.rs  
pub mod your_tool;
pub use your_tool::*;
```

4. Update `src/bin/generate_schema.rs` to include your tool:
```rust
use aws_lambda_mcp::tools::{weather, your_tool};

fn main() {
    let tools = vec![
        build_tool_from_fn("get_weather", weather::GET_WEATHER_METADATA, ...),
        build_tool_from_fn("your_tool_name", your_tool::YOUR_TOOL_NAME_METADATA, ...),
    ];
    // ...
}
```

5. Run `make schema` to regenerate schemas
6. Update handler to route to your tool

## Configuration

### Environment Variables

Create a `.env` file for local development (see `.env.example`).

### Lambda Configuration

Lambda settings are defined in `Cargo.toml`:
```toml
[package.metadata.lambda.deploy]
memory = 128
timeout = 30
tracing = "active"
```

## CI/CD Guidelines

See [AGENTS.md](./AGENTS.md) for detailed coding conventions and guidelines for AI coding agents and contributors.

See [COPILOT.md](./COPILOT.md) for GitHub Copilot-specific instructions and architectural decisions.

Key principles:
- No `unsafe` code
- No `unwrap()`, `expect()`, or `panic!()`
- Structured tracing for all operations
- Comprehensive error handling with `anyhow`
- Strict clippy lints enforced
- Custom `#[tool]` macro, not rmcp SDK

## Dependencies

Core dependencies:
- `lambda_runtime` - AWS Lambda runtime
- `tokio` - Async runtime
- `serde` / `serde_json` - Serialization
- `schemars` - JSON Schema generation
- `reqwest` - HTTP client with middleware tracing
- `tracing` - Structured logging
- `anyhow` - Error handling
- `aws-lambda-mcp-macros` - Custom `#[tool]` attribute macro

## License

[Your License Here]

## Contributing

1. Read [AGENTS.md](./AGENTS.md) for coding standards
2. Run `make lint` before committing
3. Ensure `make release` builds successfully
4. Regenerate schema with `make schema` if models change