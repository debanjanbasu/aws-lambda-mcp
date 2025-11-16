# AI Assistant Instructions

**Last updated**: 2025-11-12T00:00:00.000Z

Repository-specific instructions for AI coding assistants (GitHub Copilot, Claude, MCP agents) working on this Model Context Protocol server project.

---

## Project Overview

**Amazon Bedrock AgentCore Gateway** - Rust-based Model Context Protocol server implementation that translates Amazon Bedrock AgentCore tool calls to actual implementations. Focus: security, performance (ARM64/Graviton), minimal cold start, structured tracing.

✅ **MCP Server Implementation** - This is a full Model Context Protocol server using Amazon Bedrock AgentCore as the transport layer. We use `rmcp`'s `#[tool]` macro for MCP-compliant schema generation.

⚠️ **Critical Naming** - Always use "Amazon Bedrock AgentCore Gateway" (not "Bedrock Gateway" or "Bedrock Agent Gateway"). This is the official AWS service name.

**Stack**: Rust 2024 | Lambda Runtime | Tokio | serde/schemars | tracing | reqwest | cargo-lambda | UPX

**License**: MIT

---

## Quick Reference

### MCP Inspector
```bash
# Get help on current MCP Inspector options
npx @modelcontextprotocol/inspector --help

# Launch with HTTP transport
# Note: For HTTP transport, authentication token must be entered in the UI
npx @modelcontextprotocol/inspector \
  --transport http \
  --server-url "https://..."

# The Inspector will open in your browser
# Enter your Bearer token in the UI's authentication field
```

### Build Commands
```bash
make help     # Show all commands with colored output
make schema   # Generate tool_schema.json
make build    # Debug build
make release  # ARM64 production build with UPX compression (~1.3MB)
make test     # Run tests
make all      # Run tests and build release
```

### Deployment Commands
```bash
make setup-backend  # One-time backend setup (S3 + DynamoDB)
make deploy         # Build and deploy to AWS (with smart backend checking)
make tf-destroy     # Destroy infrastructure
```

### Development Commands
```bash
make test-token     # Get OAuth token + launch MCP Inspector
make test-lambda    # Test Lambda directly
make logs           # Tail CloudWatch logs
make login          # Authenticate AWS + Entra ID CLIs
make clean          # Remove tokens and backups
```

### Advanced Terraform Commands
```bash
make tf-init        # Initialize Terraform (with smart backend checking)
make tf-plan        # Plan Terraform changes
make tf-apply       # Apply Terraform changes
```

For full infrastructure commands: `cd iac && make help`

### Ephemeral PR Environments
```bash
# Manual environment deployment
gh workflow run pr-environment.yml -f action=deploy

# Manual environment destruction
gh workflow run pr-environment.yml -f action=destroy
```

### CI/CD Preferences

- **Colored Output**: For all command-line tools run in CI/CD workflows (like `cargo`, `terraform`, etc.), prefer colored output. Do not use flags like `-no-color`. The `CARGO_TERM_COLOR=always` environment variable should be set. Makefiles should use ANSI color codes for better developer experience.

**Binary Size**: Release builds are automatically compressed with UPX (`--best --lzma`), reducing size from ~3.7MB to ~1.3MB (65% reduction). This significantly improves cold start time.

**Ephemeral Environments**: Pull requests automatically get isolated test environments with unique resource names to prevent conflicts. All resources are tagged with PR information for easy identification and cleanup.

**Dependency Updates**: Dependabot automatically updates Rust dependencies, Terraform providers, and GitHub Actions. Updates are automatically tested and merged when passing.

**Terraform Provider Versioning**:
- Always use `>= major.0` for provider versions (e.g., `version = ">= 6.0"`). This ensures compatibility with the latest minor and patch versions within a major release, preventing lock file conflicts in CI/CD.

**Breaking Change Detection**: The project is configured to always use the latest versions of dependencies to catch breaking changes early. Failures in CI/CD pipelines due to breaking changes are expected and indicate that code updates are needed.

### Adding a New Tool
1. Create model in `src/models/`: `#[derive(Debug, Serialize, Deserialize, JsonSchema)]`
2. Create tool in `src/tools/`: Use `#[tool(description = "...")]` macro
3. Register in `src/bin/generate_schema.rs`: Add `tool_entry!(...)`
4. Run `make schema` to regenerate schemas

### Makefile Best Practices

**Smart Behavior Over Simplicity**
- Makefiles should implement intelligent behavior ("proper programs") rather than static commands
- Use colored output with ANSI escape codes for better developer experience. **Note:** When using `echo` commands with ANSI escape codes *inside a `bash -c '...'` block*, explicitly use `echo -e` (e.g., `echo -e "\033[1;32mSuccess!\033[0m"`) to ensure proper interpretation of escape sequences. For top-level `echo` commands, `echo` is usually sufficient.
- Implement validation checks (e.g., backend config checking) to prevent runtime errors
- Provide clear, actionable error messages with appropriate coloring
- Use dynamic help generation that extracts descriptions from command comments

**Delegation Pattern**
- The main Makefile (`Makefile`) should delegate infrastructure-related commands to the IAC Makefile (`iac/Makefile`)
- Use `cd iac && $(MAKE) command` pattern for proper delegation
- Maintain consistent naming conventions across both Makefiles

**Color Coding Guidelines**
- Use `\033[1;36m` for headers and section titles
- Use `\033[1;32m` for success messages and completed actions
- Use `\033[1;34m` for informational messages and ongoing actions
- Use `\033[1;33m` for warnings and important notes
- Use `\033[1;31m` for errors and critical failures
- Always reset colors with `\033[0m` after colored text

**Dynamic Help System**
- Use `grep` and `awk` to dynamically extract help text from command comments
- Follow the pattern: `command: ## Description of what this command does`
- Group related commands in help output with clear section headers

**Backend Configuration Checking**
- Implement smart checks for required configuration files (e.g., `iac/backend.config`)
- Provide clear guidance on how to resolve missing configuration issues
- Prevent operations that would fail due to missing prerequisites

---

## Code Standards

### ✅ Always Do
- Return `Result<T>` for fallible operations, use `?` with `.context()`
- Use `#[instrument]` for tracing spans
- Add `#[must_use]` to pure functions
- Document errors with `# Errors` sections
- Run `cargo clippy -- -D warnings` before finishing
- Use field-level `#[schemars(description = "...")]` for API documentation
- Keep functions under 60 lines
- Use explicit types for clarity

### ❌ Never Do
- `unwrap()`, `expect()`, `panic!()` → Use `Result` and `?`
- `unsafe` code → Denied by lints
- Hard-coded secrets/ARNs → Use environment variables
- Blocking I/O in async → Use async/await everywhere
- Unused imports → Clean up after refactoring
- Wildcard imports → Explicit imports only
- Integrating full rmcp SDK → Only use `#[tool]` macro

### Modern Rust Patterns
```rust
// ✅ Good
country_code.as_deref().map_or(default, func)
option.is_some_and(|x| x.is_valid())
HTTP_CLIENT.get(url)  // Direct LazyLock access
format!("{}", path.display())
let msg = format!("Hello {name}");  // Inline args

// ❌ Bad
country_code.as_ref().map(String::as_str).map_or(...)
option.map_or(false, |x| x.is_valid())
fn get_client() -> &Client { ... }  // Unnecessary wrapper
format!("{:?}", path)
format!("{}", name)  // Don't wrap single var
```

---

## Architecture

```
src/
├── main.rs               # Lambda bootstrap + tracing init
├── handler.rs            # Core Lambda handler
├── models/               # Domain types (derive JsonSchema)
│   ├── mod.rs
│   └── weather.rs
├── tools/                # Tool implementations (#[tool] macro)
│   ├── mod.rs
│   └── weather.rs
├── http/                 # Global HTTP_CLIENT
│   ├── mod.rs
│   └── client.rs
└── bin/
    └── generate_schema.rs  # Schema generator
```

**Models**: Derive `JsonSchema` for auto-schema generation  
**Tools**: Annotated with `#[tool(description = "...")]`  
**Schema**: Auto-generated from rmcp metadata + schemars types → Bedrock format

---

## Tool Development

### 1. Define Models
```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WeatherRequest {
    #[schemars(description = "City, address, or place name")]
    pub location: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct WeatherResponse {
    pub location: String,
    pub temperature: f64,
    pub temperature_unit: TemperatureUnit,
}
```

### 2. Implement Tool
```rust
use rmcp::tool;
use tracing::instrument;

/// Get current weather information.
///
/// # Errors
/// Returns error if geocoding or API call fails.
#[tool(description = "Get weather for a location with temperature in local units")]
#[instrument(fields(location = %request.location))]
pub async fn get_weather(request: WeatherRequest) -> Result<WeatherResponse> {
    let data = HTTP_CLIENT
        .get(url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .context("Failed to fetch weather")?
        .json()
        .await?;
    
    Ok(WeatherResponse { /* ... */ })
}
```

### 3. Register Tool
Edit `src/bin/generate_schema.rs`:
```rust
let tools = vec![
    tool_entry!(
        aws_lambda_mcp::tools::weather::get_weather_tool_attr(),
        aws_lambda_mcp::models::WeatherRequest,
        aws_lambda_mcp::models::WeatherResponse
    ),
    // Your new tool here
];
```

---

## Error Handling

```rust
use anyhow::{Context, Result};

async fn fetch_data(url: &str) -> Result<Data> {
    HTTP_CLIENT
        .get(url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .context("Failed to fetch data")?
        .json()
        .await
        .context("Failed to parse response")
}
```

**Pattern**: Use `.context()` for actionable error messages, avoid redundant context that repeats function names.

---

## Tracing

```rust
use tracing::{debug, info, instrument};

#[instrument(fields(user_id = %id))]
async fn process(id: UserId, data: Data) -> Result<Response> {
    debug!(url = %api_url, "Fetching data");
    
    let result = do_work(&data).await?;
    
    info!(
        duration_ms = result.duration.as_millis(),
        status = %result.status,
        "Request completed"
    );
    
    Ok(result)
}
```

**Init**: JSON logging for CloudWatch (in `main.rs`)  
**HTTP**: Auto-traced via `reqwest-tracing` middleware  
**Fields**: Use structured key-value pairs

---

## Performance

- Minimize allocations: Prefer `&str` over `String`
- Avoid cloning large structs
- Keep dependencies minimal (cold start impact)
- Use `LazyLock` for one-time initialization
- Per-request timeouts: `.timeout(Duration::from_secs(n))`
- Mark compile-time functions as `const fn`

---

## rmcp Integration

### What We Use
- `rmcp`'s `#[tool]` macro for metadata extraction
- Automatic `{function_name}_tool_attr()` generation
- Integration with `schemars` for JSON Schema
- MCP-compliant tool schema generation

### What We Don't Use
- MCP server infrastructure (ServerHandler, ToolRouter)
- MCP transports (stdio/SSE/HTTP)
- Full MCP protocol implementation

### Why Custom Approach?
1. **Different transport layer**: Amazon Bedrock AgentCore ≠ Standard MCP transports
2. **Lambda runtime**: Optimized for serverless execution
3. **Performance**: Minimal dependencies for fast cold start
4. **Schema format**: Amazon Bedrock AgentCore specific format requirements

---

## Schema Generation

**Process**:
1. `rmcp` macro generates `{function}_tool_attr()` accessor with MCP-compliant metadata
2. `generate_schema` binary calls these accessors
3. `schemars` generates JSON Schema from request/response types
4. Custom cleanup transforms to Amazon Bedrock AgentCore format (no `$schema`, inline enums)

**Run**: `make schema` before building

**Amazon Bedrock AgentCore Format**:
- No `$schema` or `$defs`
- Enums converted to `string` type
- No complex nested types

This generates a valid Model Context Protocol schema that is compatible with Amazon Bedrock AgentCore Gateway.

---

## Security

- ❌ No secrets in code → Use env vars or Secrets Manager
- ❌ No authentication logic in AI code → Human review required
- ❌ No unsafe/unwrap/panic → Enforced by lints
- ✅ Input validation on all external data
- ✅ Structured logging without PII
- ✅ MIT/Apache-2.0/BSD licensed dependencies only

---

## Review Checklist

Before marking task complete:

- [ ] Compiles without warnings: `cargo clippy --all-targets -- -D warnings`
- [ ] Formatted: `cargo fmt`
- [ ] Schema regenerated: `make schema`
- [ ] Release build succeeds: `make release`
- [ ] No `unwrap/expect/panic/unsafe`
- [ ] Error contexts are actionable
- [ ] Tracing spans on async operations
- [ ] Public functions have `#[must_use]` if pure
- [ ] Fallible functions document `# Errors`
- [ ] No unused imports or dead code
- [ ] No wildcard imports or match arms
- [ ] Makefiles have colored output and smart behavior
- [ ] Makefile help systems work correctly
- [ ] Backend config checking prevents errors

---

## Common Pitfalls

1. **Don't use rmcp SDK fully** - Only the `#[tool]` macro
2. **Don't change to MCP schema** - Must be Amazon Bedrock AgentCore format
3. **Don't call it "Bedrock Gateway"** - Always use full name "Amazon Bedrock AgentCore Gateway"
4. **Don't oversimplify Makefiles** - Treat them as "proper programs" with intelligent behavior and colored output
5. **Don't skip backend config checking** - Always implement smart validation to prevent runtime errors
6. **Don't use static help text** - Use dynamic help generation with `grep` and `awk`

---

## When Unsure

Leave a TODO comment for human review:
```rust
// TODO: Human review needed for authentication logic
// TODO: Verify this timeout value for production
```

**Principle**: Simple, direct, secure code > Clever abstractions

---

## GitHub Template Repository Setup

When this repository is used as a GitHub template, several secrets need to be configured for the GitHub Actions workflows to function properly.

### Required GitHub Secrets

- `AWS_IAM_ROLE_ARN` - AWS IAM Role ARN for GitHub Actions OIDC authentication
- `AZURE_CLIENT_ID` - Entra ID App Registration Client ID  
- `AZURE_TENANT_ID` - Entra ID Tenant ID
- `TF_BACKEND_BUCKET` - S3 Bucket name for Terraform state storage
- `TF_BACKEND_DYNAMODB_TABLE` - DynamoDB Table name for Terraform state locking

### Optional GitHub Secrets


### Setup Process

1. Configure OIDC authentication between GitHub and AWS/Entra ID
2. Run `make setup-backend` locally to create Terraform backend resources
3. Set the required secrets in your repository's Settings > Secrets and variables > Actions

Refer to the main README.md for detailed setup instructions.

---

## Future Extensibility

Planned additions (don't implement without discussion):
- `src/secrets.rs` - AWS Secrets Manager integration
- `src/cache.rs` - ElastiCache integration
- Additional tools in `src/tools/`

Keep modular boundaries clear for future expansion.

---

**Remember**: Human maintainers have final authority on all changes.
