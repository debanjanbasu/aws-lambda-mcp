# Copilot Instructions

This file provides context and instructions for GitHub Copilot when working on this project.

## Project Purpose

This is an AWS Lambda function that serves as a **Bedrock Agent gateway** - it translates tool calls from AWS Bedrock to actual tool implementations. It is NOT a traditional MCP (Model Context Protocol) server.

## Architecture Decisions

### Why Not Use rmcp SDK?

The `rmcp` (Rust Model Context Protocol SDK) at https://github.com/modelcontextprotocol/rust-sdk is excellent for building MCP servers, but we intentionally chose NOT to use it because:

1. **Different Use Case**:
   - `rmcp` is for building MCP servers that communicate via stdio/SSE/HTTP transports
   - We're building an AWS Lambda function that integrates with Bedrock Agents
   - Bedrock has its own schema format and requirements

2. **Schema Incompatibility**:
   - MCP tools use one schema format
   - AWS Bedrock Agent tools use a different format (no enums, specific structure)
   - We need custom schema generation for Bedrock

3. **Unnecessary Complexity**:
   - `rmcp` includes routing, transport layers, and server infrastructure
   - We only need tool metadata and schema generation
   - Lambda runtime handles the HTTP/invocation layer

4. **Performance**:
   - Every dependency adds to cold start time
   - Minimal binary size is critical for Lambda
   - Custom approach keeps it lean

### What We Learned from rmcp

While we don't use `rmcp` directly, we learned valuable patterns:

1. **Macro Design**: Using `#[tool(description = "...")]` attribute macro for clean metadata
2. **Schema from Types**: Leveraging `schemars::JsonSchema` for automatic schema generation
3. **Doc Comments**: Could extract descriptions from `///` comments (not implemented yet)
4. **Annotations**: Could add hints like `read_only`, `idempotent` (not implemented yet)

## Tool Development Pattern

### 1. Define Models (in `src/models/`)

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WeatherRequest {
    /// Location name (city, address, or place)
    pub location: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct WeatherResponse {
    pub location: String,
    pub temperature: f64,
    // ...
}
```

**Key points**:
- Input types: `Deserialize + JsonSchema`
- Output types: `Serialize + JsonSchema`
- Use `///` doc comments - they appear in generated schema
- Keep types simple (Bedrock doesn't support complex enums)

### 2. Implement Tool (in `src/tools/`)

```rust
use crate::macros::tool;
use crate::models::{WeatherRequest, WeatherResponse};

#[tool(description = "Get current weather information for a specified location")]
#[instrument(fields(location = %request.location))]
pub async fn get_weather(request: WeatherRequest) -> Result<WeatherResponse> {
    // Implementation
}
```

**Key points**:
- Use `#[tool(description = "...")]` from our custom macro
- Add `#[instrument]` for tracing
- Return `Result<T>` for proper error handling
- Tool name = function name (auto-derived)

### 3. Generate Schema

```bash
make schema  # Runs: cargo run --bin generate-schema
```

This reads the compiled code and generates `tool_schema.json` with Bedrock-compatible format.

### 4. Build for Lambda

```bash
make release  # Runs: cargo lambda build --release --arm64
```

## Code Style Guidelines

### ✅ DO

```rust
// Use as_deref() instead of as_ref().map()
country_code.as_deref().map_or(default, func)

// Use is_some_and() for boolean checks
option.is_some_and(|x| x.is_valid())

// Direct access to LazyLock globals
HTTP_CLIENT.get(url)

// Use Display for paths
format!("{}", path.display())

// Inline format args
let msg = format!("Hello {name}");  // not format!("{}", name)
```

### ❌ DON'T

```rust
// Don't chain unnecessarily
value.as_ref().map(String::as_str).map_or(default, func)

// Don't use map_or for boolean checks
option.map_or(false, |x| x.is_valid())

// Don't create getter wrappers
fn get_client() -> &'static Client { &HTTP_CLIENT }

// Don't use Debug for paths
format!("{:?}", path)

// Don't use rmcp SDK
use rmcp::tool;  // NO! Use crate::macros::tool
```

## Error Handling

Always use `anyhow::Result` with context:

```rust
use anyhow::{Context, Result};

async fn fetch_data(url: &str) -> Result<Data> {
    HTTP_CLIENT
        .get(url)
        .send()
        .await
        .context("Failed to fetch data")?
        .json()
        .await
        .context("Failed to parse response")
}
```

## Tracing

Use structured logging with meaningful fields:

```rust
use tracing::{debug, info, instrument};

#[instrument(fields(user_id = %id, request_type))]
async fn process_request(id: UserId, data: RequestData) -> Result<Response> {
    debug!("Starting processing");
    
    let result = do_work(&data).await?;
    
    info!(
        duration_ms = result.duration.as_millis(),
        status = %result.status,
        "Request completed"
    );
    
    Ok(result)
}
```

## HTTP Client

We have a global `HTTP_CLIENT` with tracing enabled:

```rust
use crate::http::HTTP_CLIENT;
use std::time::Duration;

let response = HTTP_CLIENT
    .get(url)
    .timeout(Duration::from_secs(10))  // Per-request timeout
    .send()
    .await?;
```

**Why global?**
- Reuses connection pools
- Configured once with tracing
- Simpler than passing around
- Set timeout per-request, not globally

## Testing

When adding tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_weather_api() {
        let request = WeatherRequest {
            location: "San Francisco".to_string(),
        };
        
        let response = get_weather(request).await;
        assert!(response.is_ok());
    }
}
```

## Performance Tips

1. **Minimize allocations**: Use `&str` when ownership not needed
2. **Avoid clones**: Pass references when possible
3. **Keep dependencies minimal**: Every crate adds to cold start
4. **Use `const fn`**: Compile-time evaluation when possible
5. **LazyLock for globals**: One-time initialization

## Security Checklist

- ✅ No hard-coded secrets, ARNs, tokens
- ✅ No `unwrap()`, `expect()`, `panic!()`
- ✅ No `unsafe` code
- ✅ Input validation on all external data
- ✅ Structured logging (no PII in logs)

## Build & Deploy

```bash
# Generate schema from code
make schema

# Run clippy (required before commit)
cargo clippy --all-targets -- -D warnings

# Format code
cargo fmt

# Build for ARM64/Graviton
make release

# The binary is at: target/lambda/aws-lambda-mcp/bootstrap
```

## Future Enhancements

Ideas for later (don't implement without discussion):

1. **Doc comment extraction**: Parse `///` comments instead of `description = "..."`
2. **Tool annotations**: Add `read_only`, `idempotent`, `destructive` hints
3. **Auto-derive request/response types**: Parse function signature in macro
4. **Secrets integration**: AWS Secrets Manager client
5. **Caching layer**: ElastiCache integration

## Questions to Ask

Before implementing a feature, consider:

1. Does this increase binary size or cold start time?
2. Is there a simpler way without adding dependencies?
3. Does this complicate the code unnecessarily?
4. Can we leverage existing Rust patterns instead?
5. Is this actually needed for AWS Lambda/Bedrock?

## Common Mistakes to Avoid

1. **Using rmcp SDK** - We have custom tools, don't integrate full MCP
2. **Changing schema format** - Must follow Bedrock Agent requirements
3. **Adding unwrap/expect** - Always use `?` with `Result`
4. **Blocking I/O** - Use async/await for all I/O
5. **Over-abstracting** - Keep it simple, direct code is better

## Useful Links

- [AWS Bedrock Agent Documentation](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/)
- [cargo-lambda Documentation](https://www.cargo-lambda.info/)
- [schemars Documentation](https://docs.rs/schemars/)
- [rmcp SDK Reference](https://github.com/modelcontextprotocol/rust-sdk) (for patterns only)

---

When in doubt, refer to `AGENTS.md` for detailed coding guidelines.
