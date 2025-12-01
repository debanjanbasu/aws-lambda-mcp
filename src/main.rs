use aws_lambda_mcp::handler::function_handler;
use lambda_runtime::{Error, service_fn};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Use tracing subscriber for CloudWatch Logs
    tracing_subscriber::fmt::init();

    lambda_runtime::run(service_fn(function_handler)).await
}
