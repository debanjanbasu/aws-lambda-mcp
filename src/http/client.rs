use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::{SpanBackendWithUrl, TracingMiddleware};
use std::sync::LazyLock;

pub static HTTP_CLIENT: LazyLock<ClientWithMiddleware> = LazyLock::new(|| {
    let reqwest_client = Client::new();

    ClientBuilder::new(reqwest_client)
        .with(TracingMiddleware::<SpanBackendWithUrl>::new())
        .build()
});
