use async_trait::async_trait;
use reqwest::Client;
use std::sync::LazyLock;

/// Trait for HTTP client operations to enable testing with mocks.
///
/// This trait abstracts HTTP operations to allow dependency injection
/// for testing purposes, preventing tests from making real network calls.
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Send a GET request and return the raw response.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    async fn get(&self, url: &str) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>>;

    /// Send a GET request and return the JSON response as Value.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or JSON parsing fails.
    async fn get_json_value(&self, url: &str) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>;
}

/// Production HTTP client implementation using reqwest.
///
/// This client wraps `reqwest::Client` to implement the `HttpClient` trait
/// for production use.
pub struct ReqwestClient {
    client: Client,
}

impl ReqwestClient {
    #[must_use]
    pub const fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn get(&self, url: &str) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.client.get(url).send().await?)
    }

    async fn get_json_value(&self, url: &str) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client.get(url).send().await?;
        let json = response.json().await?;
        Ok(json)
    }
}

/// Global HTTP client for production use
pub static HTTP_CLIENT: LazyLock<ReqwestClient> = LazyLock::new(|| {
    ReqwestClient::new(Client::new())
});
