# WMTP Protocol Specification

**Version:** 0.1.0  
**Transport:** WebTransport (QUIC)  
**Encryption:** TLS 1.3 (mandatory)

## Overview

WMTP (WebTransport Mail Transfer Protocol) is a modern email transfer protocol built on QUIC/WebTransport, designed for low-latency, secure, real-time mail communication.

## Connection

### Endpoint
wss://api.wmtp.online:443 (Production)
wss://localhost:4433 (Development)


### Transport
- Protocol: WebTransport over HTTP/3
- Encryption: TLS 1.3 (always on)
- Streams: Bidirectional (control) + Unidirectional (data)

## Message Format

All messages are JSON-encoded UTF-8 strings.

### Request Format
```json
{
  "cmd": "COMMAND_NAME",
  "data": {
    "field1": "value1",
    "field2": "value2"
  }
}


## Response Format
{
  "status": "OK | ERR",
  "cmd": "RESPONSE_TYPE",
  "msg": "Optional message",
  "session_token": "token (if applicable)",
  "authenticated": true | false,
  "code": 1001,
  "data": {}
}


Commands
Session Commands
INIT
Initialize a new session. Request:
json
{ "cmd": "INIT" }
Response:
json
{
  "status": "OK",
  "cmd": "SESSION_INIT",
  "session_token": "WMTP-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "authenticated": false
}
AUTH
Authenticate with email. Request:
json
{
  "cmd": "AUTH",
  "data": { "email": "user@example.com" }
}
Response:
json
{
  "status": "OK",
  "cmd": "AUTH_OK",
  "session_token": "64-char-hmac-token",
  "authenticated": true,
  "email": "user@example.com",
  "username": "user"
}
RESUME
Resume existing session. Request:
json
{
  "cmd": "RESUME",
  "data": { "token": "session-token" }
}
LOGOUT
End session. Request:
json
{
  "cmd": "LOGOUT",
  "data": { "token": "session-token" }
}
Connectivity Commands
PING
Test connectivity. Request:
json
{ "cmd": "PING" }
Response:
json
{ "status": "OK", "cmd": "PONG" }
HB (Heartbeat)
Server sends periodically to keep connection alive.
json
{ "cmd": "HB", "ts": 1732608000 }
Info Commands
STATUS
Get server status. Request:
json
{ "cmd": "STATUS" }
Response:
json
{
  "status": "OK",
  "cmd": "STATUS",
  "data": {
    "active_sessions": 42,
    "server": "wmtp-server",
    "version": "0.1.0"
  }
}
INFO
Get server info. Request:
json
{ "cmd": "INFO" }
Error Codes
Code	Description
1001	Malformed JSON
1002	Unknown command
1003	Missing required field
1004	Invalid format
2001	Authentication failed
2002	Authentication required
2003	Session not found
2004	Session expired
3001	Mail not found
3002	Mailbox not found
3003	Recipient not found
5000	Internal server error
Future Commands (Planned)
SEND - Send mail
FETCH - Fetch mail
LIST - List mailbox
DELETE - Delete mail
SEARCH - Search messages
Security
All connections use TLS 1.3
Session tokens are HMAC-SHA256 based
Tokens are deterministic per email+secret
No plaintext credentials transmitted
text

---

### **10. docs/API.md**

```markdown
# WMTP API Reference

## JavaScript Client API

### WMTPTransport

Connection management class.

#### Methods

##### `connect(url)`
Connect to WMTP server.
```javascript
await wmtpTransport.connect("https://localhost:4433");
disconnect()
Disconnect from server.
javascript
await wmtpTransport.disconnect();
send(data)
Send data to server.
javascript
await wmtpTransport.send({ cmd: "PING" });
isConnected()
Check connection status.
javascript
if (wmtpTransport.isConnected()) { ... }
Events
onConnect - Called when connected
onDisconnect - Called when disconnected
onMessage - Called when message received
onError - Called on error
WMTPProtocol
Protocol handler class.
Methods
init()
Initialize session.
javascript
await wmtpProtocol.init();
auth(email)
Authenticate with email.
javascript
await wmtpProtocol.auth("user@example.com");
resume(token)
Resume session with token.
javascript
await wmtpProtocol.resume("session-token");
logout()
End session.
javascript
await wmtpProtocol.logout();
ping()
Ping server.
javascript
await wmtpProtocol.ping();
status()
Get server status.
javascript
await wmtpProtocol.status();
info()
Get server info.
javascript
await wmtpProtocol.info();
getSession()
Get current session info.
javascript
const session = wmtpProtocol.getSession();
// { token, authenticated, email, username }
Events
onSessionInit - Session initialized
onAuthSuccess - Authentication successful
onHeartbeat - Heartbeat received
onResponse - Response received
onError - Error occurred
Usage Example
javascript
// Connect
await wmtpTransport.connect("https://localhost:4433");

// Initialize session
await wmtpProtocol.init();

// Authenticate
await wmtpProtocol.auth("user@wmtp.online");

// Check status
await wmtpProtocol.status();

// Logout
await wmtpProtocol.logout();

// Disconnect
await wmtpTransport.disconnect();
text

---

### **11. docs/SETUP.md**

```markdown
# WMTP Setup Guide

## Prerequisites

- Rust 1.70+ (for server)
- OpenSSL 3.x (for certificates)
- Modern browser (Chrome 97+, Edge 97+, Firefox 114+)

## Quick Start

### 1. Clone Repository
```bash
git clone https://github.com/yourusername/wmtp.git
cd wmtp
2. Generate Certificates
bash
cd certs
openssl genpkey -algorithm RSA -out key.pem
openssl req -new -x509 -key key.pem -out cert.pem -days 365 -subj "/CN=localhost"
cd ..
3. Build Server
bash
cd server
cargo build --release
4. Configure Server
Edit server/.env:
text
WMTP_HOST=0.0.0.0
WMTP_PORT=4433
WMTP_SERVER_SECRET=your-secret-key-here
WMTP_CERT_PATH=../certs/cert.pem
WMTP_KEY_PATH=../certs/key.pem
5. Run Server
bash
cargo run --release
6. Open Client
Open client/app.html in your browser. Note: For self-signed certificates, you may need to:
Navigate to https://localhost:4433 in your browser
Accept the security warning
Then use the client app
Production Deployment
Server Setup (VPS)
Install Rust:
bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
Clone and build:
bash
git clone https://github.com/yourusername/wmtp.git
cd wmtp/server
cargo build --release
Get SSL certificate (Let's Encrypt):
bash
sudo apt install certbot
sudo certbot certonly --standalone -d api.wmtp.online