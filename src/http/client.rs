use reqwest::Client;
use std::sync::LazyLock;

/// Global HTTP client
pub static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::new()
});
