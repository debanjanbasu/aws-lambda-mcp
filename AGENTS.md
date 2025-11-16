# AI Assistant Instructions for AWS Lambda MCP

**Build/Lint/Test Commands:**
- `make build` - Debug build
- `make release` - ARM64 production build with UPX compression
- `make test` - Run all tests
- `cargo test <test_name>` - Run single test
- `cargo clippy -- -D warnings` - Lint with strict warnings
- `cargo fmt` - Format code
- `make schema` - Generate tool schemas

**Code Style Guidelines:**
- **Error Handling**: Return `Result<T>`, use `?` with `.context()`, no `unwrap/expect/panic`
- **Imports**: Explicit imports only, no wildcards, clean up unused imports
- **Types**: Use explicit types for clarity, derive `Debug, Serialize, Deserialize, JsonSchema`
- **Naming**: `snake_case` for variables/functions, `PascalCase` for types, `UPPERCASE` for constants
- **Functions**: Under 60 lines, `#[must_use]` on pure functions, document `# Errors`
- **Async**: Use `async/await` everywhere, no blocking I/O, `#[instrument]` for tracing
- **Security**: No unsafe code, no hardcoded secrets, environment variables only
- **Performance**: Prefer `&str` over `String`, minimize allocations, `LazyLock` for globals
