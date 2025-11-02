# AGENTS.md

Last updated: 2025-11-02T12:04:47.000Z

This file provides repository-specific instructions for AI coding agents (GitHub Copilot, Copilot Chat, MCP-aware agents) to ensure secure, consistent, and high-quality contributions.

## 1. Project Overview
Rust-based AWS Lambda function serving as a Bedrock Agent gateway. Translates tool calls from AWS Bedrock to actual tool implementations. Focused on security, performance (arm64/Graviton), structured tracing, and minimal binary size.

**Important**: This is NOT a traditional MCP server. It's a Lambda function that implements the AWS Bedrock Agent gateway pattern. We use `rmcp`'s `#[tool]` macro for metadata extraction, but not the full rmcp server infrastructure, because:
- Bedrock Agent has different schema requirements than MCP
- We don't need MCP server infrastructure (stdio/SSE/HTTP transports, tool routing, etc.)
- We leverage rmcp's tool metadata and schemars integration for schema generation
- We transform rmcp's standard schemas to AWS Bedrock format

Primary goals: predictable latency, low cold start time, denial of unsafe code, clean JSON I/O.

## 2. Tech Stack
Language: Rust (edition 2024).
Runtime: AWS Lambda (lambda_runtime crate, built with cargo-lambda).
Async: Tokio.
Serialization: serde / serde_json.
Schema Generation: schemars for JSON Schema + custom schema generator.
Observability: tracing + tracing-subscriber (JSON, env-filter).
Error Handling: anyhow (no unwrap/expect/panic).
HTTP Client: reqwest with reqwest-middleware and reqwest-tracing.
Build Tool: cargo-lambda for ARM64/Graviton deployment.
Tool Metadata: rmcp's `#[tool]` macro for tool registration and metadata.

**Note on rmcp**: We use `rmcp` and `rmcp-macros` for tool annotation and metadata extraction, but NOT for server infrastructure (no ServerHandler, ToolRouter, or transport layers). We extract rmcp's tool metadata and transform schemas to AWS Bedrock format via our schema generator.

## 3. Coding Conventions
- No unsafe blocks (enforced by lints).
- No unwrap(), expect(), panic! (clippy deny).
  - ❌ Replace `panic!()` with `eprintln!() + process::exit(1)` for early exit scenarios.
- Prefer Result<T, anyhow::Error> for fallible operations.
- Use `?` for propagation, attach context with `anyhow::Context` when meaningful.
- Functions: keep < 60 LOC; split if larger.
- Use explicit types over generic inference when clarity aids agents.
- Data models in `models/` should derive: `#[derive(Debug, Serialize, Deserialize, JsonSchema)]` for API types.
- Tracing: use spans for long-running async tasks; structured fields: `tracing::info!(request_id, event_type, "..." );`.
- **Add `#[must_use]` to pure functions**: Functions that return computed values without side effects should be marked `#[must_use]`.
- **Remove unused imports**: Always clean up unused imports after refactoring.

## 3.1. Code Clarity & Simplicity
- **Avoid unnecessary method chains**: Prefer direct, readable code over chained conversions.
  - ❌ BAD: `geo_result.country_code.as_ref().map(String::as_str).map_or(TemperatureUnit::C, TemperatureUnit::from_country_code)`
  - ✅ GOOD: `geo_result.country_code.as_deref().map_or(TemperatureUnit::C, TemperatureUnit::from_country_code)`
  - ✅ BETTER: Match patterns directly when clearer
- **Use symbols and operators judiciously**: Prioritize readability over clever use of `?`, turbofish, or complex type annotations.
- **Avoid over-abstracting**: Don't introduce traits, generics, or helper functions unless they're reused 3+ times.
- **Direct access over getters**: Use `HTTP_CLIENT` directly instead of `get_client()` wrappers.
- **Use `is_some_and()` over `map_or(false, ...)`**: Modern Rust pattern for boolean checks.
  - ❌ BAD: `opt.map_or(false, |x| x.is_async())`
  - ✅ GOOD: `opt.is_some_and(|x| x.is_async())`
- **Collapse nested `if let` chains**: Use `&&` patterns for cleaner matching.
  - ❌ BAD: `if let Some(x) = a { if let Some(y) = b { ... } }`
  - ✅ GOOD: `if let (Some(x), Some(y)) = (a, b) { ... }`
- **Use `.display()` for paths instead of `{:?}`**: Paths should use Display trait.
  - ❌ BAD: `format!("{:?}", path)`
  - ✅ GOOD: `format!("{}", path.display())`
- **Inline format args**: Don't use `format!("{}", var)`, use direct interpolation.
  - ❌ BAD: `format!("{}", name)`
  - ✅ GOOD: Just `name` or `name.clone()` if ownership needed
- **Use explicit match arms**: Avoid wildcard `_` when specific variants exist.
  - ❌ BAD: `_ => unreachable!()`
  - ✅ GOOD: `ReturnType::Default => unreachable!()`

## 4. Project Structure
```
src/
├── main.rs          - Lambda bootstrap & tracing init & lib exports
├── handler.rs       - Core Lambda handler logic (event -> response)
├── macros/          - Re-export tool macro from macros crate
│   └── mod.rs       - pub use aws_lambda_mcp_macros::tool;
├── models/          - Domain models organized by feature
│   ├── mod.rs       - Re-exports
│   └── weather.rs   - Weather-specific types (with JsonSchema)
├── tools/           - Business logic functions (annotated with #[tool])
│   ├── mod.rs       - Re-exports
│   └── weather.rs   - Weather API integration
├── http/            - HTTP client setup
│   ├── mod.rs       - Re-exports
│   └── client.rs    - Global HTTP_CLIENT with tracing
└── bin/
    └── generate_schema.rs - Schema generator (uses main crate as lib)

macros/              - Proc macro crate for #[tool] attribute
├── Cargo.toml       - Proc-macro library
└── src/
    └── lib.rs       - Tool attribute macro implementation
```

The project is both a binary (Lambda function) and library (for schema generation).
Models in `models/` derive `JsonSchema` for automatic schema generation.
Tool functions use `#[tool(description = "...")]` macro for metadata export.

## 5. Performance Guidance
- Minimize allocations: prefer &str over String where owning not required.
- Avoid cloning large structs; pass references.
- Keep dependency count minimal to reduce cold start and binary size.
- Use LazyLock for one-time global initialization.
- Set request-specific timeouts via `.timeout(Duration::from_secs(n))` rather than global client config.
- **Use `const fn` where possible**: Functions that can be evaluated at compile time should be marked `const`.

## 6. Security & Compliance
- Never introduce hard-coded secrets, ARNs, tokens, tenant IDs.
- Use AWS Secrets Manager for secret retrieval (future integration placeholder).
- Authentication & authorization logic must be reviewed manually; agents should NOT fabricate cryptographic approaches.
- Avoid adding crates with ambiguous or copyleft licenses; prefer permissive (MIT/Apache-2.0/BSD). Flag any new dependency explicitly in PR description.

## 7. Dependency Policy
- Add dependencies only if they materially improve performance, security, or maintainability.
- Avoid heavy frameworks; prefer focused crates.
- Do not add macros generating large implicit code unless necessary.
- Use default features unless specific features are required (e.g., reqwest with rustls-tls).

## 8. Error Handling Pattern
Example:
```rust
pub async fn process(event: Input) -> anyhow::Result<Output> {
    let value = compute(&event)
        .context("Failed to compute value")?;
    Ok(Output { value })
}
```
- Add context only when it aids diagnosis.
- Keep context messages concise and actionable.
- Avoid redundant context that repeats function names.
- **Add `# Errors` doc sections**: All fallible public functions should document error conditions.
  ```rust
  /// Fetches weather data.
  ///
  /// # Errors
  /// Returns error if geocoding or weather API fails, or if location is invalid.
  pub async fn get_weather(request: WeatherRequest) -> Result<WeatherResponse>
  ```

## 9. Tracing Pattern
Initialization (main.rs): JSON layer + env filter for CloudWatch.
In functions:
```rust
#[instrument(fields(location = %request.location))]
pub async fn get_weather(request: WeatherRequest) -> Result<WeatherResponse> {
    debug!(url = %geocoding_url, "Fetching data");
    // ...
}
```
- Use `#[instrument]` for function-level spans.
- Use `debug!`, `info!`, `warn!`, `error!` for structured logging.
- HTTP requests/responses automatically traced via reqwest-tracing middleware.

## 10. HTTP Client Usage
- Use `HTTP_CLIENT` from `crate::http::HTTP_CLIENT` directly.
- Set per-request timeouts: `HTTP_CLIENT.get(url).timeout(Duration::from_secs(10))`.
- Client is lazily initialized with reqwest-tracing middleware for automatic request/response logging.

## 10.1. Tool Definitions & Schema Generation
### Tool Macro
- Mark tool functions with `#[rmcp::tool(description = "...")]` attribute macro from rmcp SDK.
- The macro generates a `{function_name}_tool_attr()` function that exports tool metadata.
- Leverage rmcp's integration with schemars for automatic JSON Schema generation.
- Example:
  ```rust
  #[rmcp::tool(description = "Get weather for a location")]
  #[instrument(fields(location = %request.location))]
  pub async fn get_weather(request: WeatherRequest) -> Result<WeatherResponse> {
      // implementation
  }
  ```
- Still include `/// # Errors` doc comments for clippy compliance.
- Tool name is always the function name (auto-derived by rmcp).
- **USE `#[rmcp::tool]`** - it integrates with our schemars-based models.

### Integration with rmcp
- We use rmcp's `#[tool]` macro for metadata extraction
- Rmcp provides `{function_name}_tool_attr()` accessors we use in schema generation
- Rmcp integrates with schemars (which our models already use)
- We don't use rmcp's ServerHandler, ToolRouter, or transport layers
- Our `generate_schema` binary transforms rmcp metadata + schemars schemas → AWS Bedrock format

### Schema Generation
- Tool schemas are automatically generated from code via `generate_schema` binary.
- Schema generator calls `{tool}_tool_attr()` functions generated by rmcp's `#[tool]` macro.
- Uses schemars to generate JSON schemas from request/response models.
- Transforms standard JSON Schema → AWS Bedrock format (removes $schema, inlines enums as strings).
- Models must derive `JsonSchema` from schemars crate.
- Run `make schema` to regenerate `tool_schema.json` before building.
- Schema format follows AWS Bedrock Agent requirements (no enums, inlined types).
- **Schema is NOT MCP-compatible** - it's specifically for AWS Bedrock Agents.

### Why Not Use rmcp SDK?
The `rmcp` (Rust MCP SDK) is designed for building Model Context Protocol servers that:
- Communicate via stdio, SSE, or HTTP transports
- Use MCP's tool/prompt/resource protocol
- Have routing and handler infrastructure

Our use case is different:
- We're building an AWS Lambda function
- Bedrock Agent has its own schema format (different from MCP)
- We need minimal overhead and cold start time
- Custom approach is simpler and more maintainable

We use `rmcp`'s tool annotation and metadata extraction directly, but not the server infrastructure.

### Tool Structure
```
src/tools/
└── weather.rs  - Annotated with #[rmcp::tool(...)]
    ├── get_weather()  - Function with rmcp tool macro
    └── get_weather_tool_attr()  - Auto-generated by rmcp (returns ToolAttr)
src/models/
└── weather.rs  - Request/Response types with #[derive(JsonSchema)]
src/bin/
└── generate_schema.rs - Calls tool_attr() functions, uses schemars, generates tool_schema.json
```


## 11. Testing & Validation (future expansion)
Currently minimal. When tests are added:
- Use #[tokio::test] for async.
- Prefer deterministic mocks over network calls.
Agents should scaffold tests but mark TODO for external integrations.

## 12. Prompting Guidelines For Agents
Good prompts:
- "Refactor handler.rs to reduce nesting without adding unwrap/expect."
- "Add tracing spans for long-running operations without altering logic."
- "Simplify this chain: make it more readable."
Bad prompts:
- "Insert secret for API key."
- "Bypass auth checks."

## 13. Review Checklist (apply to AI output)
- Compiles with current toolchain.
- No unsafe / unwrap / expect / panic.
- Maintains tracing consistency.
- Dependencies unchanged or justified.
- Error messages informative but not leaking PII/secrets.
- Code is readable and doesn't over-use method chaining or clever tricks.
- **Run `cargo clippy --all-targets -- -D warnings`** and fix all warnings before completing task.
- Public functions returning values have `#[must_use]`.
- Fallible functions have `# Errors` documentation.
- No unused imports or variables.
- Variable names are distinct (avoid `req_type`/`res_type` style ambiguity, use `request_type`/`response_type`).
- Run `make schema` to ensure tool_schema.json is up to date.
- Run `make release` to verify Lambda builds successfully for ARM64.

## 14. Things Agents MUST NOT DO
- Invent security-critical algorithms or credentials.
- Introduce dynamic code execution features.
- Add blocking I/O inside async contexts without justification.
- Over-engineer simple code with unnecessary abstractions.
- Use `as_ref().map()` when `as_deref()` is clearer.
- Leave unused imports or dead code.
- Use wildcard imports in production code (e.g., `use crate::*;`).
- Use wildcard match arms `_` when specific variants exist.
- Submit code without running `cargo clippy`.
- **Try to integrate rmcp's ServerHandler or ToolRouter** - we only use rmcp's tool metadata, not server infrastructure.
- **Change schema format to MCP** - we must follow Bedrock Agent requirements.
- **Remove rmcp::tool macro** - we depend on it for metadata extraction.

## 15. Future Extensibility Notes
- Secrets retrieval module will live in `src/secrets.rs` (planned).
- Caching integration may add `src/cache.rs` referencing ElastiCache.
- New tools should be added to `src/tools/` with appropriate models in `src/models/`.
Agents should keep modular boundaries clear for future additions.

## 16. If Unsure
Prefer leaving a clearly marked TODO comment: `// TODO: human review needed for ...` rather than guessing.

## 17. Summary (Quick Reference)
Focus: secure, performant Rust Lambda with clean, readable code.
Build: Use `cargo lambda build --release --arm64` for production ARM64/Graviton deployment.
Avoid: unsafe, unwrap/expect/panic, secrets, unnecessary deps, over-abstraction, clever code, unused imports, wildcard imports.
Always: structured tracing, contextual errors, small functions, direct access patterns, run clippy before completion, regenerate schema, use #[rmcp::tool] macro for tools.
Prefer: simplicity over cleverness, as_deref() over as_ref().map(), direct symbol access, is_some_and() over map_or(false, ...).
Document: `#[must_use]` on pure functions, `# Errors` on fallible functions, tool descriptions in #[tool] macro.
Modern patterns: inline format args, .display() for paths, const fn, collapsed if-let chains.
Schema: Models derive JsonSchema, tools use #[rmcp::tool] macro, schemas auto-generated from rmcp metadata via generate_schema binary, tool name = function name.

---
These instructions guide automated agents; human maintainers have final authority.
