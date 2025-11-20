use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::{SpanBackendWithUrl, TracingMiddleware};
use std::sync::LazyLock;

/// Global HTTP client with tracing middleware
pub static HTTP_CLIENT: LazyLock<ClientWithMiddleware> = LazyLock::new(|| {
    let reqwest_client = Client::new();

    ClientBuilder::new(reqwest_client)
        .with(TracingMiddleware::<SpanBackendWithUrl>::new())
        .build()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_initialization() {
        // The LazyLock should initialize the client
        let _client = &*HTTP_CLIENT;
        // If we get here without panicking, the client was initialized successfully
    }
}
