use aws_lambda_mcp::handler::function_handler;
use lambda_runtime::{Error, service_fn};
use std::io::stdout;
use std::mem::drop;
use tracing_appender::non_blocking;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(function_handler);

    let (writer, log_guard) = non_blocking(stdout());
    lambda_runtime::tracing::init_default_subscriber_with_writer(writer);

    let shutdown_hook = || async move {
        drop(log_guard);
    };
    lambda_runtime::spawn_graceful_shutdown_handler(shutdown_hook).await;

    lambda_runtime::run(func).await
}
