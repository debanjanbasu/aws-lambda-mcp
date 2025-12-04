//! Custom error types for the AWS Lambda MCP application.
//!
//! This module defines error types that are specific to the application's domain,
//! providing more meaningful error information to users and making error handling
//! more precise.

use std::fmt;

/// Custom error type for the application.
#[derive(Debug)]
pub enum AppError {
    /// Error related to geocoding operations
    GeocodingError(String),
    /// Error related to weather API operations
    WeatherApiError(String),
    /// Error related to user information extraction
    UserExtractionError(String),
    /// Generic error for other cases
    GenericError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GeocodingError(msg) => write!(f, "Geocoding error: {msg}"),
            Self::WeatherApiError(msg) => write!(f, "Weather API error: {msg}"),
            Self::UserExtractionError(msg) => write!(f, "User extraction error: {msg}"),
            Self::GenericError(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        Self::GenericError(error.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        Self::GenericError(error.to_string())
    }
}