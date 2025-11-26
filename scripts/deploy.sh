#!/bin/bash

# WMTP Deployment Script
# Usage: ./deploy.sh [server|client|all]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}   WMTP Deployment Script${NC}"
echo -e "${GREEN}================================${NC}"

# Configuration
SERVER_USER="root"
SERVER_HOST="your-server-ip"
SERVER_PATH="/opt/wmtp"
BRANCH="main"

deploy_server() {
    echo -e "${YELLOW}Deploying server...${NC}"
    
    # Build locally
    cd server
    cargo build --release
    
    # Copy to server
    scp target/release/wmtp-server ${SERVER_USER}@${SERVER_HOST}:${SERVER_PATH}/
    scp .env ${SERVER_USER}@${SERVER_HOST}:${SERVER_PATH}/
    
    # Restart service
    ssh ${SERVER_USER}@${SERVER_HOST} "systemctl restart wmtp"
    
    echo -e "${GREEN}Server deployed!${NC}"
}

deploy_client() {
    echo -e "${YELLOW}Deploying client...${NC}"
    
    # Push to GitHub (Cloudflare Pages auto-deploys)
    cd client
    git add .
    git commit -m "Deploy: $(date +%Y-%m-%d_%H:%M:%S)" || true
    git push origin ${BRANCH}
    
    echo -e "${GREEN}Client deployed to Cloudflare Pages!${NC}"
}

case "$1" in
    server)
        deploy_server
        ;;
    client)
        deploy_client
        ;;
    all)
        deploy_server
        deploy_client
        ;;
    *)
        echo "Usage: $0 {server|client|all}"
        exit 1
        ;;
esac

echo -e "${GREEN}Deployment complete!${NC}"
