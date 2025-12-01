use aws_lambda_mcp::handler::function_handler;
use lambda_runtime::{Error, service_fn};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Use Lambda runtime's built-in tracing subscriber for CloudWatch Logs
    lambda_runtime::tracing::init_default_subscriber();

    lambda_runtime::run(service_fn(function_handler)).await
}
