#!/bin/bash

# WMTP Server Setup Script
# Run on a fresh Ubuntu/Debian VPS

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}   WMTP Server Setup${NC}"
echo -e "${GREEN}================================${NC}"

# Update system
echo -e "${YELLOW}Updating system...${NC}"
apt update && apt upgrade -y

# Install dependencies
echo -e "${YELLOW}Installing dependencies...${NC}"
apt install -y build-essential pkg-config libssl-dev curl git

# Install Rust
echo -e "${YELLOW}Installing Rust...${NC}"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Create directory
echo -e "${YELLOW}Creating directories...${NC}"
mkdir -p /opt/wmtp
cd /opt/wmtp

# Clone repository
echo -e "${YELLOW}Cloning repository...${NC}"
git clone https://github.com/yourusername/wmtp.git .

# Build server
echo -e "${YELLOW}Building server...${NC}"
cd server
cargo build --release

# Install certbot
echo -e "${YELLOW}Installing certbot...${NC}"
apt install -y certbot

# Get SSL certificate
echo -e "${YELLOW}Getting SSL certificate...${NC}"
read -p "Enter domain (e.g., api.wmtp.online): " DOMAIN
certbot certonly --standalone -d $DOMAIN

# Create .env
echo -e "${YELLOW}Creating configuration...${NC}"
cat > .env << EOF
WMTP_HOST=0.0.0.0
WMTP_PORT=443
WMTP_DOMAIN=${DOMAIN}
WMTP_SERVER_SECRET=$(openssl rand -hex 32)
WMTP_CERT_PATH=/etc/letsencrypt/live/${DOMAIN}/fullchain.pem
WMTP_KEY_PATH=/etc/letsencrypt/live/${DOMAIN}/privkey.pem
WMTP_SESSION_TIMEOUT=3600
WMTP_HEARTBEAT_INTERVAL=5
RUST_LOG=info,wmtp_server=debug
EOF

# Create systemd service
echo -e "${YELLOW}Creating systemd service...${NC}"
cat > /etc/systemd/system/wmtp.service << EOF
[Unit]
Description=WMTP Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/wmtp/server
ExecStart=/opt/wmtp/server/target/release/wmtp-server
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
echo -e "${YELLOW}Starting service...${NC}"
systemctl daemon-reload
systemctl enable wmtp
systemctl start wmtp

# Setup auto-renewal
echo -e "${YELLOW}Setting up certificate renewal...${NC}"
echo "0 0 1 * * certbot renew --quiet && systemctl restart wmtp" | crontab -

# Firewall
echo -e "${YELLOW}Configuring firewall...${NC}"
ufw allow 443/tcp
ufw allow 443/udp

echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}   Setup Complete!${NC}"
echo -e "${GREEN}================================${NC}"
echo ""
echo -e "Server running at: https://${DOMAIN}"
echo -e "Check status: systemctl status wmtp"
echo -e "View logs: journalctl -u wmtp -f"
