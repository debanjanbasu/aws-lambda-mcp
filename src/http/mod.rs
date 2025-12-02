use std::sync::LazyLock;
use reqwest::Client;

pub static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);
// Note: The original project likely had more complex client setup (e.g., TLS config).
// This is a minimal replacement based on the dependency.
