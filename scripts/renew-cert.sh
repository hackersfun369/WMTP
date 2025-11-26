#!/bin/bash

# Renew Let's Encrypt certificates and restart WMTP

certbot renew --quiet

if [ $? -eq 0 ]; then
    systemctl restart wmtp
    echo "Certificates renewed and WMTP restarted"
else
    echo "Certificate renewal failed"
    exit 1
fi
