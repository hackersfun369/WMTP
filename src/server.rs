//! WMTP Server - Main server logic
//! 
//! Handles WebTransport/QUIC connections and WMTP protocol commands.

use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

use wtransport::{Endpoint, Identity, ServerConfig};
use wtransport::endpoint::IncomingSession;
use wtransport::stream::{RecvStream, SendStream};

use crate::commands::{cmd, Heartbeat, Request, Response};
use crate::config::Config;
use crate::error::codes;
use crate::session::{create_session_store, SessionStore, WmtpSession};
use crate::token::{generate_ephemeral_token, generate_identity_token};

/// Run the WMTP server
pub async fn run_server() -> Result<()> {
    // Load configuration
    let config = Config::from_env();

    // Validate configuration
    if let Err(e) = config.validate() {
        error!("❌ Configuration error: {}", e);
        return Err(anyhow::anyhow!(e));
    }

    // Log configuration
    info!("📋 Configuration:");
    info!("   Host: {}", config.host);
    info!("   Port: {}", config.port);
    info!("   Domain: {}", config.domain);
    info!("   Cert: {:?}", config.cert_path);
    info!("   Key: {:?}", config.key_path);
    info!("   Session timeout: {}s", config.session_timeout);
    info!("   Heartbeat interval: {}s", config.heartbeat_interval);

    // Load TLS identity from certificate and key
    let identity = Identity::load_pemfiles(&config.cert_path, &config.key_path).await?;
    info!("🔐 TLS identity loaded successfully");

    // Build server configuration
    let server_config = ServerConfig::builder()
        .with_bind_default(config.port)
        .with_identity(&identity)
        .build();

    // Create WebTransport endpoint
    let endpoint = Endpoint::server(server_config)?;
    
    info!("");
    info!("🚀 WMTP Server is running!");
    info!("   WebTransport URL: https://{}:{}", config.domain, config.port);
    info!("   Local URL: https://localhost:{}", config.port);
    info!("");
    info!("⏳ Waiting for connections...");
    info!("");

    // Create session store (in-memory)
    let sessions: SessionStore = create_session_store();
    
    // Store server secret
    let server_secret = Arc::new(config.server_secret.clone());
    let heartbeat_interval = config.heartbeat_interval;

    // Main accept loop
    loop {
        // Wait for incoming connection
        let incoming: IncomingSession = endpoint.accept().await;
        
        // Clone for async task
        let sessions = sessions.clone();
        let secret = server_secret.clone();

        // Spawn handler task
        tokio::spawn(async move {
            if let Err(e) = handle_connection(incoming, sessions, secret, heartbeat_interval).await {
                error!("❌ Connection error: {:?}", e);
            }
        });
    }
}

/// Handle an incoming WebTransport connection
async fn handle_connection(
    incoming: IncomingSession,
    sessions: SessionStore,
    secret: Arc<String>,
    hb_interval: u64,
) -> Result<()> {
    // Accept the session request
    let session_request = incoming.await?;
    
    // Accept the connection
    let connection = session_request.accept().await?;
    let connection = Arc::new(connection);

    info!("✅ New client connected");

    // Accept the first bidirectional stream as the control stream
    let (send, recv) = connection.accept_bi().await?;
    info!("📡 Control stream established");

    // Spawn handlers for additional streams
    spawn_bidi_handler(connection.clone());
    spawn_uni_handler(connection.clone());

    // Handle the control stream (main protocol loop)
    if let Err(e) = handle_control_stream(send, recv, sessions, secret, hb_interval).await {
        warn!("⚠️ Control stream ended: {:?}", e);
    }

    info!("👋 Client disconnected");
    Ok(())
}

/// Handle the control stream - main WMTP command processing loop
async fn handle_control_stream(
    mut send: SendStream,
    mut recv: RecvStream,
    sessions: SessionStore,
    secret: Arc<String>,
    hb_interval: u64,
) -> Result<()> {
    // Setup heartbeat timer
    let mut heartbeat = interval(Duration::from_secs(hb_interval));
    let mut buf = [0u8; 8192];

    loop {
        tokio::select! {
            // Send heartbeat periodically
            _ = heartbeat.tick() => {
                let hb = Heartbeat::new().to_json();
                if send.write_all(hb.as_bytes()).await.is_err() {
                    warn!("💔 Heartbeat failed, closing connection");
                    break;
                }
            }

            // Read incoming data
            result = recv.read(&mut buf) => {
                match result {
                    Ok(Some(n)) if n > 0 => {
                        // Parse as UTF-8
                        let text = match std::str::from_utf8(&buf[..n]) {
                            Ok(t) => t.trim(),
                            Err(_) => {
                                warn!("⚠️ Received non-UTF8 data, ignoring");
                                continue;
                            }
                        };

                        info!("📥 Received: {}", text);

                        // Process the command and get response
                        let response = process_command(text, &sessions, &secret).await;
                        
                        info!("📤 Sending: {}", response);

                        // Send response
                        if send.write_all(response.as_bytes()).await.is_err() {
                            warn!("⚠️ Failed to send response");
                            break;
                        }
                    }
                    Ok(Some(_)) => {
                        // Zero bytes read
                        break;
                    }
                    Ok(None) => {
                        // Stream closed by peer
                        info!("📪 Control stream closed by peer");
                        break;
                    }
                    Err(e) => {
                        warn!("⚠️ Read error: {:?}", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Process a WMTP command and return JSON response
async fn process_command(text: &str, sessions: &SessionStore, secret: &str) -> String {
    // Parse the request
    let req = match Request::from_json(text) {
        Ok(r) => r,
        Err(e) => {
            return Response::err("PARSE", &format!("Invalid JSON: {}", e), codes::MALFORMED_JSON)
                .to_json();
        }
    };

    let command = req.cmd.to_uppercase();

    match command.as_str() {
        // ==================== INIT ====================
        // Initialize a new session
        cmd::INIT => {
            let token = generate_ephemeral_token();
            
            // Store session (release lock before await)
            {
                let mut store = sessions.lock().unwrap();
                store.insert(token.clone(), WmtpSession::new_ephemeral(token.clone()));
            }

            Response::ok("SESSION_INIT")
                .with_token(token)
                .with_auth(false)
                .with_msg("Session initialized")
                .to_json()
        }

        // ==================== AUTH ====================
        // Authenticate with email
        cmd::AUTH => {
            let email = match req.get_str("email") {
                Some(e) if !e.trim().is_empty() => e.trim().to_lowercase(),
                _ => {
                    return Response::err("AUTH", "Missing or empty email", codes::MISSING_FIELD)
                        .to_json();
                }
            };

            // Validate email format (basic check)
            if !email.contains('@') || !email.contains('.') {
                return Response::err("AUTH", "Invalid email format", codes::INVALID_FORMAT)
                    .to_json();
            }

            // Generate permanent identity token
            let token = generate_identity_token(&email, secret);
            let username = email.split('@').next().unwrap_or("user").to_string();

            // Store authenticated session
            {
                let mut store = sessions.lock().unwrap();
                store.insert(
                    token.clone(),
                    WmtpSession::new_authenticated(token.clone(), email.clone()),
                );
            }

            Response::ok("AUTH_OK")
                .with_token(token)
                .with_auth(true)
                .with_email(email)
                .with_username(username)
                .with_msg("Authentication successful")
                .to_json()
        }

        // ==================== RESUME ====================
        // Resume an existing session
        cmd::RESUME => {
            let token = match req.get_str("token") {
                Some(t) if !t.trim().is_empty() => t,
                _ => {
                    return Response::err("RESUME", "Missing token", codes::MISSING_FIELD)
                        .to_json();
                }
            };

            // Look up session
            let session = {
                let mut store = sessions.lock().unwrap();
                if let Some(s) = store.get_mut(&token) {
                    s.touch(); // Update last activity
                    Some(s.clone())
                } else {
                    None
                }
            };

            match session {
                Some(s) => {
                    let mut resp = Response::ok("SESSION_RESUMED")
                        .with_token(s.token)
                        .with_auth(s.authenticated);

                    if let Some(email) = s.email {
                        resp = resp.with_email(email);
                    }
                    if let Some(username) = s.username {
                        resp = resp.with_username(username);
                    }

                    resp.to_json()
                }
                None => {
                    Response::err("RESUME", "Session not found", codes::SESSION_NOT_FOUND)
                        .to_json()
                }
            }
        }

        // ==================== LOGOUT ====================
        // End a session
        cmd::LOGOUT => {
            if let Some(token) = req.get_str("token") {
                let mut store = sessions.lock().unwrap();
                store.remove(&token);
            }

            Response::ok("LOGOUT_OK")
                .with_msg("Logged out successfully")
                .to_json()
        }

        // ==================== PING ====================
        // Simple ping/pong
        cmd::PING => {
            Response::ok("PONG")
                .with_msg("pong")
                .to_json()
        }

        // ==================== STATUS ====================
        // Get server status
        cmd::STATUS => {
            let (total, authenticated) = {
                let store = sessions.lock().unwrap();
                let total = store.len();
                let auth = store.values().filter(|s| s.authenticated).count();
                (total, auth)
            };

            Response::ok("STATUS")
                .with_data(serde_json::json!({
                    "active_sessions": total,
                    "authenticated_sessions": authenticated,
                    "server": "wmtp-server",
                    "version": env!("CARGO_PKG_VERSION")
                }))
                .to_json()
        }

        // ==================== INFO ====================
        // Get server info
        cmd::INFO => {
            Response::ok("INFO")
                .with_data(serde_json::json!({
                    "name": "WMTP Server",
                    "version": env!("CARGO_PKG_VERSION"),
                    "protocol": "WebTransport/QUIC",
                    "domain": "wmtp.online",
                    "features": ["session", "auth", "heartbeat"]
                }))
                .to_json()
        }

        // ==================== UNKNOWN ====================
        _ => {
            Response::err(
                "UNKNOWN",
                &format!("Unknown command: {}", command),
                codes::UNKNOWN_COMMAND,
            )
            .to_json()
        }
    }
}

/// Spawn handler for additional bidirectional streams (echo)
fn spawn_bidi_handler(conn: Arc<wtransport::Connection>) {
    tokio::spawn(async move {
        loop {
            match conn.accept_bi().await {
                Ok((mut send, mut recv)) => {
                    tokio::spawn(async move {
                        let mut buf = [0u8; 4096];
                        while let Ok(Some(n)) = recv.read(&mut buf).await {
                            if n == 0 {
                                break;
                            }
                            info!("📨 Bidi stream: {} bytes", n);
                            // Echo back
                            if send.write_all(&buf[..n]).await.is_err() {
                                break;
                            }
                        }
                        info!("📭 Bidi stream closed");
                    });
                }
                Err(_) => break,
            }
        }
        info!("🔚 Bidi accept loop ended");
    });
}

/// Spawn handler for unidirectional streams (receive only)
fn spawn_uni_handler(conn: Arc<wtransport::Connection>) {
    tokio::spawn(async move {
        loop {
            match conn.accept_uni().await {
                Ok(mut recv) => {
                    tokio::spawn(async move {
                        let mut buf = [0u8; 4096];
                        let mut total = 0usize;
                        while let Ok(Some(n)) = recv.read(&mut buf).await {
                            if n == 0 {
                                break;
                            }
                            total += n;
                        }
                        info!("📨 Uni stream closed: {} bytes received", total);
                    });
                }
                Err(_) => break,
            }
        }
        info!("🔚 Uni accept loop ended");
    });
}
