use aws_lambda_mcp::handler::function_handler;
use lambda_runtime::{Error, run, service_fn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configure tracing for CloudWatch Logs - AWS Lambda handles JSON formatting
    // Use default formatter for best compatibility with Lambda logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        // Use environment variable RUST_LOG with INFO as default
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        // Disable ANSI colors for CloudWatch compatibility
        .with_ansi(false)
        // Use AWS Lambda's built-in timestamps
        .without_time()
        // Reduce log verbosity by removing module paths
        .with_target(false)
        .init();

    run(service_fn(function_handler)).await
}
