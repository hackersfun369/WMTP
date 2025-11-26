//! Identity token generation and verification using HMAC-SHA256
//! 
//! Provides deterministic, permanent identity tokens based on email.

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Generate a deterministic identity token from email + server secret
/// 
/// # Arguments
/// * `email` - User's email address
/// * `server_secret` - Server's secret key for HMAC
/// 
/// # Returns
/// Hex-encoded HMAC token (64 characters)
/// 
/// # Example
/// ```
/// let token = generate_identity_token("user@example.com", "secret");
/// assert_eq!(token.len(), 64);
/// ```
pub fn generate_identity_token(email: &str, server_secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(server_secret.as_bytes())
        .expect("HMAC can take key of any size");
    
    // Normalize email to lowercase for consistency
    let normalized_email = email.trim().to_lowercase();
    mac.update(normalized_email.as_bytes());
    
    let result = mac.finalize().into_bytes();
    hex::encode(result)
}

/// Verify a deterministic identity token against an email
/// 
/// # Arguments
/// * `token` - Token to verify
/// * `email` - Email to verify against
/// * `server_secret` - Server's secret key
/// 
/// # Returns
/// `true` if token is valid for given email
pub fn verify_identity_token(token: &str, email: &str, server_secret: &str) -> bool {
    let expected = generate_identity_token(email, server_secret);
    constant_time_eq(&expected, token)
}

/// Generate a temporary ephemeral session token (non-deterministic)
/// 
/// Used for sessions before user authenticates with email.
pub fn generate_ephemeral_token() -> String {
    format!("WMTP-{}", uuid::Uuid::new_v4())
}

/// Check if a token is ephemeral (temporary)
pub fn is_ephemeral_token(token: &str) -> bool {
    token.starts_with("WMTP-")
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    
    result == 0
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_deterministic() {
        let email = "test@example.com";
        let secret = "test-secret-key";
        
        let token1 = generate_identity_token(email, secret);
        let token2 = generate_identity_token(email, secret);
        
        // Same input should produce same token
        assert_eq!(token1, token2);
        assert_eq!(token1.len(), 64); // SHA256 hex = 64 chars
    }

    #[test]
    fn test_token_different_emails() {
        let secret = "test-secret-key";
        
        let token1 = generate_identity_token("alice@example.com", secret);
        let token2 = generate_identity_token("bob@example.com", secret);
        
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_token_case_insensitive() {
        let secret = "test-secret-key";
        
        let token1 = generate_identity_token("Test@Example.COM", secret);
        let token2 = generate_identity_token("test@example.com", secret);
        
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_token_verification() {
        let email = "test@example.com";
        let secret = "test-secret-key";
        
        let token = generate_identity_token(email, secret);
        
        assert!(verify_identity_token(&token, email, secret));
        assert!(!verify_identity_token(&token, "wrong@example.com", secret));
        assert!(!verify_identity_token("invalid-token", email, secret));
    }

    #[test]
    fn test_ephemeral_token() {
        let token = generate_ephemeral_token();
        assert!(is_ephemeral_token(&token));
        assert!(!is_ephemeral_token("regular-token"));
        assert!(token.starts_with("WMTP-"));
    }
}
