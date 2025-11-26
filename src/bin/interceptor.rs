use aws_lambda_mcp::interceptor::{process_interceptor_request, InterceptorEvent, InterceptorResponse};
use lambda_runtime::{Error, LambdaEvent};
use tracing::{info, instrument};

#[instrument(skip(event))]
async fn interceptor_handler(event: LambdaEvent<InterceptorEvent>) -> Result<InterceptorResponse, Error> {
    Ok(process_interceptor_request(event.payload))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configure tracing for CloudWatch Logs
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .init();

    info!("Starting Bedrock AgentCore Gateway interceptor");

    lambda_runtime::run(lambda_runtime::service_fn(interceptor_handler)).await
}