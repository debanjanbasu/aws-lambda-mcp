use aws_lambda_mcp::handler::function_handler;
use lambda_runtime::{Error, run, service_fn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configure tracing for CloudWatch Logs compatibility
    tracing_subscriber::fmt()
        .json()
        // Use environment variable RUST_LOG with INFO as default
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        // Remove duplicated span information from logs
        .with_current_span(false)
        // Disable ANSI colors for CloudWatch compatibility
        .with_ansi(false)
        // Use AWS Lambda's built-in timestamps
        .without_time()
        // Reduce log verbosity by removing module paths
        .with_target(false)
        .init();

    run(service_fn(function_handler)).await
}
