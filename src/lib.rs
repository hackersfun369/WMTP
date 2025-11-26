//! WMTP Server Library
//! 
//! WebTransport Mail Transfer Protocol implementation in Rust.
//! Built on QUIC for secure, low-latency mail transfer.

pub mod config;
pub mod commands;
pub mod error;
pub mod server;
pub mod session;
pub mod token;

// Re-exports for convenience
pub use config::Config;
pub use error::{WmtpError, WmtpResult};
pub use server::run_server;
pub use session::{SessionStore, WmtpSession, create_session_store};
pub use token::{generate_identity_token, verify_identity_token, generate_ephemeral_token};
