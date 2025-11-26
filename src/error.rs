//! Custom error types for WMTP server

use thiserror::Error;

/// WMTP-specific error types
#[derive(Error, Debug)]
pub enum WmtpError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("TLS/Certificate error: {0}")]
    Tls(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Internal server error: {0}")]
    Internal(String),
}

/// Result type alias for WMTP operations
pub type WmtpResult<T> = Result<T, WmtpError>;

/// Error codes for protocol responses
pub mod codes {
    // Parse errors (1xxx)
    pub const MALFORMED_JSON: u32 = 1001;
    pub const UNKNOWN_COMMAND: u32 = 1002;
    pub const MISSING_FIELD: u32 = 1003;
    pub const INVALID_FORMAT: u32 = 1004;
    
    // Auth errors (2xxx)
    pub const AUTH_FAILED: u32 = 2001;
    pub const AUTH_REQUIRED: u32 = 2002;
    pub const SESSION_NOT_FOUND: u32 = 2003;
    pub const SESSION_EXPIRED: u32 = 2004;
    pub const INVALID_TOKEN: u32 = 2005;
    
    // Mail errors (3xxx)
    pub const MAIL_NOT_FOUND: u32 = 3001;
    pub const MAILBOX_NOT_FOUND: u32 = 3002;
    pub const RECIPIENT_NOT_FOUND: u32 = 3003;
    pub const MAIL_TOO_LARGE: u32 = 3004;
    
    // Server errors (5xxx)
    pub const INTERNAL_ERROR: u32 = 5000;
    pub const SERVICE_UNAVAILABLE: u32 = 5001;
}
