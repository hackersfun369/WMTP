/**
 * WMTP Protocol Handler
 * Handles WMTP commands and responses
 */

class WMTPProtocol {
    constructor(transport) {
        this.transport = transport;
        this.sessionToken = null;
        this.authenticated = false;
        this.email = null;
        this.username = null;
        
        this.onSessionInit = null;
        this.onAuthSuccess = null;
        this.onAuthFail = null;
        this.onHeartbeat = null;
        this.onResponse = null;
        this.onError = null;
    }

    /**
     * Initialize session
     */
    async init() {
        return this.send({ cmd: 'INIT' });
    }

    /**
     * Authenticate with email
     * @param {string} email - User email
     */
    async auth(email) {
        return this.send({
            cmd: 'AUTH',
            data: { email }
        });
    }

    /**
     * Resume session with token
     * @param {string} token - Session token
     */
    async resume(token) {
        return this.send({
            cmd: 'RESUME',
            data: { token }
        });
    }

    /**
     * Logout
     */
    async logout() {
        const result = await this.send({
            cmd: 'LOGOUT',
            data: { token: this.sessionToken }
        });
        
        this.sessionToken = null;
        this.authenticated = false;
        this.email = null;
        this.username = null;
        
        return result;
    }

    /**
     * Ping server
     */
    async ping() {
        return this.send({ cmd: 'PING' });
    }

    /**
     * Get server status
     */
    async status() {
        return this.send({ cmd: 'STATUS' });
    }

    /**
     * Get server info
     */
    async info() {
        return this.send({ cmd: 'INFO' });
    }

    /**
     * Send custom command
     * @param {object} command - Command object
     */
    async send(command) {
        await this.transport.send(command);
    }

    /**
     * Handle incoming message
     * @param {object} message - Parsed message
     */
    handleMessage(message) {
        // Handle heartbeat
        if (message.cmd === 'HB') {
            if (this.onHeartbeat) {
                this.onHeartbeat(message);
            }
            return;
        }

        // Handle responses
        switch (message.cmd) {
            case 'SESSION_INIT':
                this.sessionToken = message.session_token;
                this.authenticated = false;
                if (this.onSessionInit) {
                    this.onSessionInit(message);
                }
                break;

            case 'AUTH_OK':
                this.sessionToken = message.session_token;
                this.authenticated = true;
                this.email = message.email;
                this.username = message.username;
                if (this.onAuthSuccess) {
                    this.onAuthSuccess(message);
                }
                break;

            case 'SESSION_RESUMED':
                this.sessionToken = message.session_token;
                this.authenticated = message.authenticated;
                this.email = message.email;
                this.username = message.username;
                if (this.onSessionInit) {
                    this.onSessionInit(message);
                }
                break;

            case 'LOGOUT_OK':
                this.sessionToken = null;
                this.authenticated = false;
                this.email = null;
                this.username = null;
                break;
        }

        // Handle errors
        if (message.status === 'ERR') {
            if (this.onError) {
                this.onError(message);
            }
        }

        // General response callback
        if (this.onResponse) {
            this.onResponse(message);
        }
    }

    /**
     * Get current session info
     */
    getSession() {
        return {
            token: this.sessionToken,
            authenticated: this.authenticated,
            email: this.email,
            username: this.username
        };
    }

    /**
     * Check if authenticated
     */
    isAuthenticated() {
        return this.authenticated;
    }

    /**
     * Store session to localStorage
     */
    saveSession() {
        if (this.sessionToken) {
            localStorage.setItem('wmtp_session', JSON.stringify({
                token: this.sessionToken,
                email: this.email,
                username: this.username
            }));
        }
    }

    /**
     * Load session from localStorage
     */
    loadSession() {
        const saved = localStorage.getItem('wmtp_session');
        if (saved) {
            try {
                return JSON.parse(saved);
            } catch (e) {
                return null;
            }
        }
        return null;
    }

    /**
     * Clear saved session
     */
    clearSession() {
        localStorage.removeItem('wmtp_session');
    }
}

// Global instance
const wmtpProtocol = new WMTPProtocol(wmtpTransport);
