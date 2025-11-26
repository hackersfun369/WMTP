/**
 * WMTP UI Controller
 * Handles user interface interactions
 */

document.addEventListener('DOMContentLoaded', () => {
    // DOM Elements
    const elements = {
        // Connection
        serverUrl: document.getElementById('serverUrl'),
        connectBtn: document.getElementById('connectBtn'),
        disconnectBtn: document.getElementById('disconnectBtn'),
        connectionStatus: document.getElementById('connectionStatus'),
        
        // Auth
        emailInput: document.getElementById('emailInput'),
        authBtn: document.getElementById('authBtn'),
        userInfo: document.getElementById('userInfo'),
        
        // Commands
        initBtn: document.getElementById('initBtn'),
        pingBtn: document.getElementById('pingBtn'),
        statusBtn: document.getElementById('statusBtn'),
        infoBtn: document.getElementById('infoBtn'),
        logoutBtn: document.getElementById('logoutBtn'),
        
        // Custom
        customCmd: document.getElementById('customCmd'),
        sendBtn: document.getElementById('sendBtn'),
        
        // Console
        console: document.getElementById('console'),
        clearBtn: document.getElementById('clearBtn'),
        
        // Navigation
        navItems: document.querySelectorAll('.nav-item'),
        views: document.querySelectorAll('.view')
    };

    // ========== Console Functions ==========
    
    function log(message, type = 'info') {
        const line = document.createElement('div');
        line.className = `console-line ${type}`;
        
        const timestamp = new Date().toLocaleTimeString();
        
        if (typeof message === 'object') {
            line.innerHTML = `<span class="timestamp">[${timestamp}]</span> <pre>${JSON.stringify(message, null, 2)}</pre>`;
        } else {
            line.innerHTML = `<span class="timestamp">[${timestamp}]</span> ${message}`;
        }
        
        elements.console.appendChild(line);
        elements.console.scrollTop = elements.console.scrollHeight;
    }

    function clearConsole() {
        elements.console.innerHTML = '';
        log('Console cleared', 'info');
    }

    // ========== UI State Functions ==========

    function updateConnectionStatus(connected) {
        const dot = elements.connectionStatus.querySelector('.status-dot');
        const text = elements.connectionStatus.querySelector('.status-text');
        
        if (connected) {
            dot.className = 'status-dot connected';
            text.textContent = 'Connected';
        } else {
            dot.className = 'status-dot disconnected';
            text.textContent = 'Disconnected';
        }
    }

    function updateUserInfo(session) {
        if (session.authenticated && session.email) {
            elements.userInfo.innerHTML = `
                <span class="user-email">${session.email}</span>
                <span class="user-status">✓ Authenticated</span>
            `;
        } else if (session.token) {
            elements.userInfo.innerHTML = `
                <span class="user-status">Session active</span>
            `;
        } else {
            elements.userInfo.innerHTML = `
                <span>Not authenticated</span>
            `;
        }
    }

    function setButtonsEnabled(connected) {
        elements.disconnectBtn.disabled = !connected;
        elements.connectBtn.disabled = connected;
        elements.authBtn.disabled = !connected;
        elements.initBtn.disabled = !connected;
        elements.pingBtn.disabled = !connected;
        elements.statusBtn.disabled = !connected;
        elements.infoBtn.disabled = !connected;
        elements.logoutBtn.disabled = !connected;
        elements.sendBtn.disabled = !connected;
    }

    // ========== Transport Event Handlers ==========

    wmtpTransport.onConnect = () => {
        log('✅ Connected to server', 'success');
        updateConnectionStatus(true);
        setButtonsEnabled(true);
    };

    wmtpTransport.onDisconnect = () => {
        log('❌ Disconnected from server', 'error');
        updateConnectionStatus(false);
        setButtonsEnabled(false);
        updateUserInfo({});
    };

    wmtpTransport.onError = (error) => {
        log(`⚠️ Error: ${error.message}`, 'error');
    };

    wmtpTransport.onMessage = (message) => {
        wmtpProtocol.handleMessage(message);
    };

    // ========== Protocol Event Handlers ==========

    wmtpProtocol.onSessionInit = (msg) => {
        log('📋 Session initialized', 'success');
        log(msg, 'response');
        updateUserInfo(wmtpProtocol.getSession());
    };

    wmtpProtocol.onAuthSuccess = (msg) => {
        log('🔐 Authentication successful', 'success');
        log(msg, 'response');
        updateUserInfo(wmtpProtocol.getSession());
        wmtpProtocol.saveSession();
    };

    wmtpProtocol.onHeartbeat = (msg) => {
        // Silent heartbeat - uncomment to show
        // log('💓 Heartbeat', 'info');
    };

    wmtpProtocol.onResponse = (msg) => {
        if (msg.cmd !== 'HB') {
            log(msg, msg.status === 'OK' ? 'response' : 'error');
        }
    };

    wmtpProtocol.onError = (msg) => {
        log(`❌ Error: ${msg.msg}`, 'error');
    };

    // ========== Button Event Handlers ==========

   elements.connectBtn.addEventListener('click', async () => {
    const url = elements.serverUrl.value.trim();
    if (!url) {
        log('Please enter a server URL', 'error');
        return;
    }
    
    log(`Connecting to ${url}...`, 'info');
    
    try {
        // Set certificate hash for self-signed cert (localhost)
        if (url.includes('localhost') || url.includes('127.0.0.1')) {
            wmtpTransport.setCertificateHash('NC0Zf64kkMJWUTeL6U+88xPHt6NHQMZ37Wuk/keMG2c=');
            log('Using self-signed certificate hash', 'info');
        }
        
        await wmtpTransport.connect(url);
    } catch (error) {
        log(`Connection failed: ${error.message}`, 'error');
    }
});

    elements.disconnectBtn.addEventListener('click', async () => {
        log('Disconnecting...', 'info');
        await wmtpTransport.disconnect();
    });

    elements.authBtn.addEventListener('click', async () => {
        const email = elements.emailInput.value.trim();
        if (!email) {
            log('Please enter an email', 'error');
            return;
        }
        
        log(`Authenticating as ${email}...`, 'info');
        await wmtpProtocol.auth(email);
    });

    elements.initBtn.addEventListener('click', async () => {
        log('Initializing session...', 'info');
        await wmtpProtocol.init();
    });

    elements.pingBtn.addEventListener('click', async () => {
        log('Sending PING...', 'info');
        await wmtpProtocol.ping();
    });

    elements.statusBtn.addEventListener('click', async () => {
        log('Requesting status...', 'info');
        await wmtpProtocol.status();
    });

    elements.infoBtn.addEventListener('click', async () => {
        log('Requesting info...', 'info');
        await wmtpProtocol.info();
    });

    elements.logoutBtn.addEventListener('click', async () => {
        log('Logging out...', 'info');
        await wmtpProtocol.logout();
        wmtpProtocol.clearSession();
        updateUserInfo({});
    });

    elements.sendBtn.addEventListener('click', async () => {
        const cmdText = elements.customCmd.value.trim();
        if (!cmdText) {
            log('Please enter a command', 'error');
            return;
        }
        
        try {
            const cmd = JSON.parse(cmdText);
            log(`Sending: ${cmdText}`, 'info');
            await wmtpProtocol.send(cmd);
        } catch (e) {
            log('Invalid JSON', 'error');
        }
    });

    elements.clearBtn.addEventListener('click', clearConsole);

    // Enter key handlers
    elements.customCmd.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            elements.sendBtn.click();
        }
    });

    elements.emailInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            elements.authBtn.click();
        }
    });

    elements.serverUrl.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            elements.connectBtn.click();
        }
    });

    // ========== Navigation ==========

    elements.navItems.forEach(item => {
        item.addEventListener('click', (e) => {
            e.preventDefault();
            
            const viewId = item.dataset.view + 'View';
            
            // Update nav
            elements.navItems.forEach(nav => nav.classList.remove('active'));
            item.classList.add('active');
            
            // Update views
            elements.views.forEach(view => {
                view.classList.remove('active');
                if (view.id === viewId) {
                    view.classList.add('active');
                }
            });
        });
    });

    // ========== Initialize ==========
    
    log('🚀 WMTP Client Ready', 'info');
    log('Connect to a server to begin', 'info');
    
    // Try to restore session
    const savedSession = wmtpProtocol.loadSession();
    if (savedSession) {
        log(`Found saved session for ${savedSession.email}`, 'info');
    }
});
