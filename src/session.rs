//! Session management for WMTP server
//! 
//! Provides thread-safe session storage and management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Represents a WMTP session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WmtpSession {
    /// Unique session token
    pub token: String,
    
    /// Whether user is authenticated
    pub authenticated: bool,
    
    /// User's email (if authenticated)
    pub email: Option<String>,
    
    /// Username extracted from email
    pub username: Option<String>,
    
    /// Session creation timestamp
    #[serde(skip)]
    pub created_at: Option<Instant>,
    
    /// Last activity timestamp
    #[serde(skip)]
    pub last_activity: Option<Instant>,
}

impl WmtpSession {
    /// Create a new unauthenticated (ephemeral) session
    pub fn new_ephemeral(token: String) -> Self {
        let now = Instant::now();
        Self {
            token,
            authenticated: false,
            email: None,
            username: None,
            created_at: Some(now),
            last_activity: Some(now),
        }
    }

    /// Create a new authenticated session
    pub fn new_authenticated(token: String, email: String) -> Self {
        let now = Instant::now();
        let username = email.split('@').next().map(String::from);
        Self {
            token,
            authenticated: true,
            email: Some(email),
            username,
            created_at: Some(now),
            last_activity: Some(now),
        }
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Some(Instant::now());
    }

    /// Check if session has expired
    pub fn is_expired(&self, timeout: Duration) -> bool {
        match self.last_activity {
            Some(last) => last.elapsed() > timeout,
            None => true,
        }
    }

    /// Get session age in seconds
    pub fn age_secs(&self) -> u64 {
        self.created_at
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0)
    }

    /// Get idle time in seconds
    pub fn idle_secs(&self) -> u64 {
        self.last_activity
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0)
    }
}

/// Thread-safe session store type
pub type SessionStore = Arc<Mutex<HashMap<String, WmtpSession>>>;

/// Create a new empty session store
pub fn create_session_store() -> SessionStore {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Session manager with helper operations
pub struct SessionManager {
    store: SessionStore,
    session_timeout: Duration,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(store: SessionStore, timeout_secs: u64) -> Self {
        Self {
            store,
            session_timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Insert a new session
    pub fn insert(&self, session: WmtpSession) {
        let mut store = self.store.lock().unwrap();
        store.insert(session.token.clone(), session);
    }

    /// Get a session by token (cloned)
    pub fn get(&self, token: &str) -> Option<WmtpSession> {
        let store = self.store.lock().unwrap();
        store.get(token).cloned()
    }

    /// Check if session exists
    pub fn exists(&self, token: &str) -> bool {
        let store = self.store.lock().unwrap();
        store.contains_key(token)
    }

    /// Update session and mark activity
    pub fn touch(&self, token: &str) -> bool {
        let mut store = self.store.lock().unwrap();
        if let Some(session) = store.get_mut(token) {
            session.touch();
            true
        } else {
            false
        }
    }

    /// Authenticate a session
    pub fn authenticate(&self, token: &str, email: String) -> bool {
        let mut store = self.store.lock().unwrap();
        if let Some(session) = store.get_mut(token) {
            session.authenticated = true;
            session.username = email.split('@').next().map(String::from);
            session.email = Some(email);
            session.touch();
            true
        } else {
            false
        }
    }

    /// Remove a session
    pub fn remove(&self, token: &str) -> Option<WmtpSession> {
        let mut store = self.store.lock().unwrap();
        store.remove(token)
    }

    /// Clean up expired sessions
    pub fn cleanup_expired(&self) -> usize {
        let mut store = self.store.lock().unwrap();
        let before = store.len();
        store.retain(|_, session| !session.is_expired(self.session_timeout));
        before - store.len()
    }

    /// Get count of active sessions
    pub fn active_count(&self) -> usize {
        let store = self.store.lock().unwrap();
        store.len()
    }

    /// Get count of authenticated sessions
    pub fn authenticated_count(&self) -> usize {
        let store = self.store.lock().unwrap();
        store.values().filter(|s| s.authenticated).count()
    }

    /// List all sessions (for debugging)
    pub fn list_all(&self) -> Vec<WmtpSession> {
        let store = self.store.lock().unwrap();
        store.values().cloned().collect()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = WmtpSession::new_ephemeral("test-token".to_string());
        assert!(!session.authenticated);
        assert!(session.email.is_none());
        assert!(session.username.is_none());
        assert!(session.created_at.is_some());
    }

    #[test]
    fn test_authenticated_session() {
        let session = WmtpSession::new_authenticated(
            "auth-token".to_string(),
            "user@example.com".to_string(),
        );
        assert!(session.authenticated);
        assert_eq!(session.email, Some("user@example.com".to_string()));
        assert_eq!(session.username, Some("user".to_string()));
    }

    #[test]
    fn test_session_manager_basic() {
        let store = create_session_store();
        let manager = SessionManager::new(store, 3600);

        let session = WmtpSession::new_ephemeral("token123".to_string());
        manager.insert(session);
        
        assert!(manager.exists("token123"));
        assert!(!manager.exists("nonexistent"));
        
        let retrieved = manager.get("token123");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_session_authentication() {
        let store = create_session_store();
        let manager = SessionManager::new(store, 3600);

        let session = WmtpSession::new_ephemeral("token123".to_string());
        manager.insert(session);
        
        assert!(!manager.get("token123").unwrap().authenticated);
        
        manager.authenticate("token123", "user@example.com".to_string());
        
        let updated = manager.get("token123").unwrap();
        assert!(updated.authenticated);
        assert_eq!(updated.email, Some("user@example.com".to_string()));
        assert_eq!(updated.username, Some("user".to_string()));
    }

    #[test]
    fn test_session_counts() {
        let store = create_session_store();
        let manager = SessionManager::new(store, 3600);

        manager.insert(WmtpSession::new_ephemeral("t1".to_string()));
        manager.insert(WmtpSession::new_authenticated(
            "t2".to_string(),
            "user@test.com".to_string(),
        ));

        assert_eq!(manager.active_count(), 2);
        assert_eq!(manager.authenticated_count(), 1);
    }
}
