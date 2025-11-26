//! Server configuration management
//! 
//! Loads settings from environment variables.

use std::env;
use std::path::PathBuf;

/// Server configuration struct
#[derive(Debug, Clone)]
pub struct Config {
    /// Host address to bind to
    pub host: String,
    
    /// Port to listen on
    pub port: u16,
    
    /// Domain name (for production)
    pub domain: String,
    
    /// Secret key for HMAC token generation
    pub server_secret: String,
    
    /// Path to TLS certificate
    pub cert_path: PathBuf,
    
    /// Path to TLS private key
    pub key_path: PathBuf,
    
    /// Session timeout in seconds
    pub session_timeout: u64,
    
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            host: env::var("WMTP_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            
            port: env::var("WMTP_PORT")
                .unwrap_or_else(|_| "4433".to_string())
                .parse()
                .unwrap_or(4433),
            
            domain: env::var("WMTP_DOMAIN")
                .unwrap_or_else(|_| "localhost".to_string()),
            
            server_secret: env::var("WMTP_SERVER_SECRET")
                .unwrap_or_else(|_| {
                    tracing::warn!("⚠️  WMTP_SERVER_SECRET not set! Using insecure default.");
                    "insecure-dev-secret-change-me-in-production".to_string()
                }),
            
            cert_path: PathBuf::from(
                env::var("WMTP_CERT_PATH")
                    .unwrap_or_else(|_| "../certs/cert.pem".to_string()),
            ),
            
            key_path: PathBuf::from(
                env::var("WMTP_KEY_PATH")
                    .unwrap_or_else(|_| "../certs/key.pem".to_string()),
            ),
            
            session_timeout: env::var("WMTP_SESSION_TIMEOUT")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
            
            heartbeat_interval: env::var("WMTP_HEARTBEAT_INTERVAL")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
        }
    }

    /// Get full bind address as string
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if !self.cert_path.exists() {
            return Err(format!("Certificate not found: {:?}", self.cert_path));
        }
        
        if !self.key_path.exists() {
            return Err(format!("Private key not found: {:?}", self.key_path));
        }
        
        if self.server_secret.len() < 16 {
            return Err("Server secret must be at least 16 characters".to_string());
        }
        
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env()
    }
}
