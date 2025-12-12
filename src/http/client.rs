use crate::http::{HttpClient, ReqwestClient};
use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;

/// Global HTTP client with optimized configuration for Lambda environment.
///
/// This client is configured with:
/// - Connection timeout of 10 seconds
/// - Request timeout of 30 seconds
/// - Connection pool with max of 10 idle connections per host
/// - TCP keepalive enabled
/// - Compression support (GZIP, Brotli, Deflate)
pub static HTTP_CLIENT: LazyLock<ReqwestClient> = LazyLock::new(|| {
    // In a Lambda environment, we can safely panic on startup if the client can't be created
    // as this indicates a fundamental configuration issue
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(10)
        .tcp_keepalive(Duration::from_secs(60))
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .build()
        .unwrap_or_else(|_| Client::new());
    ReqwestClient::new(client)
});
