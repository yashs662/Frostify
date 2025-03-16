use std::fmt;

// create a error type for asset loading errors
#[derive(Debug)]
pub enum AssetError {
    NotFound,
    ImageLoadError,
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetError::NotFound => write!(f, "Asset not found"),
            AssetError::ImageLoadError => write!(f, "Failed to load image"),
        }
    }
}

/// Custom error type for authentication-related errors
#[derive(Debug)]
pub enum AuthError {
    /// Error when connecting to Spotify API
    Network(String),
    /// Error with HTTP response from Spotify
    Api(String, Option<u16>),
    /// Error related to local authentication server
    Server(String),
    /// Timeout during authentication flow
    Timeout(String),
    /// Generic authentication error
    Generic(String),
    /// Error when parsing response
    Parse(String),
    /// Error when saving/loading tokens
    Storage(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::Network(msg) => write!(f, "Network error: {}", msg),
            AuthError::Api(msg, status) => match status {
                Some(code) => write!(f, "API error ({}): {}", code, msg),
                None => write!(f, "API error: {}", msg),
            },
            AuthError::Server(msg) => write!(f, "Server error: {}", msg),
            AuthError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            AuthError::Generic(msg) => write!(f, "Authentication error: {}", msg),
            AuthError::Parse(msg) => write!(f, "Parse error: {}", msg),
            AuthError::Storage(msg) => write!(f, "Storage error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

// Conversion from String to AuthError
impl From<String> for AuthError {
    fn from(s: String) -> Self {
        AuthError::Generic(s)
    }
}

// Conversion from &str to AuthError
impl From<&str> for AuthError {
    fn from(s: &str) -> Self {
        AuthError::Generic(s.to_string())
    }
}

// Conversion from reqwest errors
impl From<reqwest::Error> for AuthError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AuthError::Timeout(err.to_string())
        } else if err.is_connect() {
            AuthError::Network(err.to_string())
        } else if let Some(status) = err.status() {
            AuthError::Api(err.to_string(), Some(status.as_u16()))
        } else {
            AuthError::Network(err.to_string())
        }
    }
}

// Conversion from std::io errors
impl From<std::io::Error> for AuthError {
    fn from(err: std::io::Error) -> Self {
        AuthError::Server(err.to_string())
    }
}

// Conversion from tokio timeout errors
impl From<tokio::time::error::Elapsed> for AuthError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        AuthError::Timeout(err.to_string())
    }
}

// Conversion from serde_json errors
impl From<serde_json::Error> for AuthError {
    fn from(err: serde_json::Error) -> Self {
        AuthError::Parse(err.to_string())
    }
}

// Conversion from keyring errors
impl From<keyring::Error> for AuthError {
    fn from(err: keyring::Error) -> Self {
        AuthError::Storage(err.to_string())
    }
}

// Conversion from UTF-8 errors
impl From<std::str::Utf8Error> for AuthError {
    fn from(err: std::str::Utf8Error) -> Self {
        AuthError::Parse(err.to_string())
    }
}
