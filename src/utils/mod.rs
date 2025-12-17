//! Utility functions shared across the application.
//!
//! This module contains common utility functions that are used in multiple
//! parts of the application to avoid code duplication.

/// Strips Bedrock Gateway prefix from tool name.
///
/// Format: `gateway-target-id___tool_name` → `tool_name`
///
/// # Arguments
///
/// * `name` - The tool name that may contain a gateway prefix
///
/// # Returns
///
/// The tool name with the gateway prefix removed, or the original name if no prefix exists.
#[must_use]
pub fn strip_gateway_prefix(name: &str) -> String {
    name.split_once("___").map_or_else(
        || name.to_string(),
        |(_, actual_name)| actual_name.to_string(),
    )
}
