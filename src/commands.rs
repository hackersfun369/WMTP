//! WMTP protocol commands and message structures
//! 
//! Defines all supported WMTP protocol commands and request/response formats.

use serde::{Deserialize, Serialize};
use chrono::Utc;

/// Incoming WMTP request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Command name (e.g., "INIT", "AUTH", "PING")
    pub cmd: String,
    
    /// Optional data payload
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

impl Request {
    /// Parse a WMTP request from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Get a string field from data
    pub fn get_str(&self, key: &str) -> Option<String> {
        self.data
            .as_ref()?
            .get(key)?
            .as_str()
            .map(|s| s.to_string())
    }

    /// Get a nested string field from data
    pub fn get_nested_str(&self, key1: &str, key2: &str) -> Option<String> {
        self.data
            .as_ref()?
            .get(key1)?
            .get(key2)?
            .as_str()
            .map(|s| s.to_string())
    }

    /// Get an integer field from data
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.data.as_ref()?.get(key)?.as_i64()
    }

    /// Get a boolean field from data
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.as_ref()?.get(key)?.as_bool()
    }
}

/// Outgoing WMTP response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Status ("OK" or "ERR")
    pub status: String,
    
    /// Command/response type
    pub cmd: String,
    
    /// Optional message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    
    /// Session token (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
    
    /// Authentication status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticated: Option<bool>,
    
    /// User email (if authenticated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    
    /// Username (if authenticated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    
    /// Error code (if error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u32>,
    
    /// Additional data payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Response {
    /// Create a success response
    pub fn ok(cmd: &str) -> Self {
        Self {
            status: "OK".to_string(),
            cmd: cmd.to_string(),
            msg: None,
            session_token: None,
            authenticated: None,
            email: None,
            username: None,
            code: None,
            data: None,
        }
    }

    /// Create an error response
    pub fn err(cmd: &str, msg: &str, code: u32) -> Self {
        Self {
            status: "ERR".to_string(),
            cmd: cmd.to_string(),
            msg: Some(msg.to_string()),
            session_token: None,
            authenticated: None,
            email: None,
            username: None,
            code: Some(code),
            data: None,
        }
    }

    // Builder methods for chaining

    /// Add session token
    pub fn with_token(mut self, token: String) -> Self {
        self.session_token = Some(token);
        self
    }

    /// Add authentication status
    pub fn with_auth(mut self, authenticated: bool) -> Self {
        self.authenticated = Some(authenticated);
        self
    }

    /// Add email
    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    /// Add username
    pub fn with_username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    /// Add message
    pub fn with_msg(mut self, msg: &str) -> Self {
        self.msg = Some(msg.to_string());
        self
    }

    /// Add data payload
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"status":"ERR","cmd":"INTERNAL","msg":"json_serialization_failed"}"#.to_string()
        })
    }

    /// Convert to JSON bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_json().into_bytes()
    }
}

/// Heartbeat message sent periodically to keep connection alive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    /// Command type (always "HB")
    pub cmd: String,
    
    /// Unix timestamp
    pub ts: i64,
}

impl Heartbeat {
    /// Create a new heartbeat message
    pub fn new() -> Self {
        Self {
            cmd: "HB".to_string(),
            ts: Utc::now().timestamp(),
        }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"cmd":"HB","ts":0}"#.to_string()
        })
    }
}

impl Default for Heartbeat {
    fn default() -> Self {
        Self::new()
    }
}

/// Command constants
pub mod cmd {
    // Session commands
    pub const INIT: &str = "INIT";
    pub const AUTH: &str = "AUTH";
    pub const RESUME: &str = "RESUME";
    pub const LOGOUT: &str = "LOGOUT";
    
    // Connectivity commands
    pub const PING: &str = "PING";
    pub const PONG: &str = "PONG";
    pub const HB: &str = "HB";
    
    // Info commands
    pub const STATUS: &str = "STATUS";
    pub const INFO: &str = "INFO";
    
    // Mail commands (future implementation)
    pub const SEND: &str = "SEND";
    pub const FETCH: &str = "FETCH";
    pub const LIST: &str = "LIST";
    pub const DELETE: &str = "DELETE";
    pub const SEARCH: &str = "SEARCH";
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_parsing() {
        let json = r#"{"cmd":"AUTH","data":{"email":"test@example.com"}}"#;
        let req = Request::from_json(json).unwrap();
        
        assert_eq!(req.cmd, "AUTH");
        assert_eq!(req.get_str("email"), Some("test@example.com".to_string()));
    }

    #[test]
    fn test_request_missing_data() {
        let json = r#"{"cmd":"PING"}"#;
        let req = Request::from_json(json).unwrap();
        
        assert_eq!(req.cmd, "PING");
        assert!(req.data.is_none());
        assert!(req.get_str("email").is_none());
    }

    #[test]
    fn test_response_builder() {
        let resp = Response::ok("AUTH_OK")
            .with_token("token123".to_string())
            .with_auth(true)
            .with_email("user@example.com".to_string())
            .with_username("user".to_string());
        
        assert_eq!(resp.status, "OK");
        assert_eq!(resp.cmd, "AUTH_OK");
        assert_eq!(resp.session_token, Some("token123".to_string()));
        assert_eq!(resp.authenticated, Some(true));
        assert_eq!(resp.email, Some("user@example.com".to_string()));
    }

    #[test]
    fn test_error_response() {
        let resp = Response::err("AUTH", "MISSING_EMAIL", 1003);
        
        assert_eq!(resp.status, "ERR");
        assert_eq!(resp.cmd, "AUTH");
        assert_eq!(resp.msg, Some("MISSING_EMAIL".to_string()));
        assert_eq!(resp.code, Some(1003));
    }

    #[test]
    fn test_heartbeat() {
        let hb = Heartbeat::new();
        let json = hb.to_json();
        
        assert!(json.contains("HB"));
        assert!(json.contains("ts"));
    }

    #[test]
    fn test_response_to_json() {
        let resp = Response::ok("PONG").with_msg("pong");
        let json = resp.to_json();
        
        assert!(json.contains("OK"));
        assert!(json.contains("PONG"));
        assert!(json.contains("pong"));
    }
}
