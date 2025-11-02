# AWS Lambda Bedrock Agent Gateway

A production-ready AWS Lambda function that serves as a gateway for Amazon Bedrock Agents, written in Rust. This provides a secure, high-performance bridge between Bedrock AI agents and custom tool implementations.

## Important: Not an MCP Server

**This is NOT a Model Context Protocol (MCP) server.** It's an AWS Lambda function specifically designed for Amazon Bedrock Agent integration. While we reference patterns from the [rust-sdk MCP implementation](https://github.com/modelcontextprotocol/rust-sdk), we use a custom tooling approach tailored for AWS Bedrock's schema requirements.

For details on why we chose this architecture, see [COPILOT.md](./COPILOT.md).

## Features

- **Rust Performance**: Compiled for ARM64/Graviton for optimal performance and cost efficiency
- **Security First**: No `unsafe` code, no `unwrap/expect/panic`, strict clippy lints enforced
- **Observability**: Structured tracing with JSON output for CloudWatch integration
- **Clean Tool Definitions**: Use `#[tool(description = "...")]` macro for clean, maintainable code
- **Auto-Generated Schemas**: Tool schemas automatically generated from code for Bedrock compatibility
- **HTTP Client**: Pre-configured `reqwest` client with middleware tracing for all requests
- **Modern Rust**: Edition 2024 with latest async/await patterns
- **Low Cold Start**: Minimal dependencies and optimized binary size for fast Lambda cold starts

## Example Tool: Weather Service

The project includes a working example of a weather tool that:
- Takes a location name (city, address, or place)
- Geocodes the location using Open-Meteo API
- Fetches current weather data
- Intelligently returns temperature in Celsius or Fahrenheit based on country
- Returns WMO weather code and wind speed

This demonstrates the complete pattern for building Bedrock Agent tools.

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

## Prerequisites

- Rust toolchain (edition 2024)
- [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda) - `cargo install cargo-lambda`

## Getting Started

### 1. Clone and Build

```bash
# Clone the repository
git clone <your-repo-url>
cd aws-lambda-mcp

# Generate tool schema
make schema

# Build for development
make build

# Build for production (ARM64/Graviton)
make release
```

### 2. Available Make Commands

```bash
make help          # Show all available commands
make schema        # Generate tool_schema.json from code
make build         # Build Lambda function (debug)
make release       # Build Lambda function (optimized, ARM64)
make test          # Run tests
make clippy        # Run clippy linter
make fmt           # Format code
make fmt-check     # Check code formatting
make lint          # Run all linters
make clean         # Clean build artifacts
```

### 3. Local Development

```bash
# Watch for changes and auto-regenerate schema
make watch

# Development mode with auto-reload
make dev
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