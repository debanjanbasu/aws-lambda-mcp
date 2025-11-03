# OAuth Flows Without Client Secrets

The Entra ID app registration supports **multiple OAuth flows that don't require hardcoded secrets**:

## Supported Flows

### 1. Authorization Code + PKCE (Recommended)
### 2. Device Code Flow (CLI/Terminal Apps)
### 3. SPA/Public Client (Browser Apps)

---

## 1. Authorization Code + PKCE Flow

**Best for**: CLI tools, desktop apps, web apps without backend

**No client secret needed!** PKCE (Proof Key for Code Exchange) eliminates the need for secrets.

### How It Works

```
1. Generate code_verifier (random string)
2. Create code_challenge = SHA256(code_verifier)
3. User authenticates in browser
4. Exchange code + code_verifier for token
```

### Example: CLI Tool

```bash
#!/bin/bash
# get-token.sh - Get OAuth token using PKCE

CLIENT_ID=$(terraform output -raw entra_app_client_id)
TENANT_ID=$(terraform output -raw entra_tenant_id)

# Generate PKCE parameters
CODE_VERIFIER=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-43)
CODE_CHALLENGE=$(echo -n "$CODE_VERIFIER" | openssl dgst -sha256 -binary | base64 | tr -d "=+/" | cut -c1-43)

# Authorization URL
AUTH_URL="https://login.microsoftonline.com/$TENANT_ID/oauth2/v2.0/authorize"
AUTH_URL="${AUTH_URL}?client_id=${CLIENT_ID}"
AUTH_URL="${AUTH_URL}&response_type=code"
AUTH_URL="${AUTH_URL}&redirect_uri=http://localhost:8080/callback"
AUTH_URL="${AUTH_URL}&scope=api://${CLIENT_ID}/.default"
AUTH_URL="${AUTH_URL}&code_challenge=${CODE_CHALLENGE}"
AUTH_URL="${AUTH_URL}&code_challenge_method=S256"

echo "Open this URL in your browser:"
echo "$AUTH_URL"
echo ""
echo "Waiting for callback on localhost:8080..."

# Start local server to receive callback
python3 << 'PYEOF'
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import urlparse, parse_qs
import json

auth_code = None

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        global auth_code
        query = urlparse(self.path).query
        params = parse_qs(query)
        auth_code = params.get('code', [None])[0]
        
        self.send_response(200)
        self.send_header('Content-type', 'text/html')
        self.end_headers()
        self.wfile.write(b'<h1>Success! You can close this window.</h1>')
    
    def log_message(self, format, *args):
        pass

server = HTTPServer(('localhost', 8080), Handler)
server.handle_request()
print(auth_code)
PYEOF

AUTH_CODE=$(python3 << 'PYEOF'
# ... (same server code)
PYEOF
)

# Exchange code for token
TOKEN_RESPONSE=$(curl -s -X POST \
  "https://login.microsoftonline.com/$TENANT_ID/oauth2/v2.0/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=$CLIENT_ID" \
  -d "code=$AUTH_CODE" \
  -d "redirect_uri=http://localhost:8080/callback" \
  -d "grant_type=authorization_code" \
  -d "code_verifier=$CODE_VERIFIER")

ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')
echo "Access Token: $ACCESS_TOKEN"

# Use token to call gateway
curl -X POST "https://your-bedrock-gateway-endpoint" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"action": "get_weather", "parameters": {"location": "Sydney"}}'
```

### Python Example

```python
import hashlib
import secrets
import base64
import webbrowser
from urllib.parse import urlencode
import http.server
import requests

class OAuthPKCE:
    def __init__(self, client_id, tenant_id):
        self.client_id = client_id
        self.tenant_id = tenant_id
        self.auth_code = None
        
    def generate_pkce_pair(self):
        """Generate PKCE code verifier and challenge"""
        code_verifier = base64.urlsafe_b64encode(secrets.token_bytes(32)).decode('utf-8')
        code_verifier = code_verifier.replace('=', '').replace('+', '-').replace('/', '_')
        
        code_challenge = hashlib.sha256(code_verifier.encode('utf-8')).digest()
        code_challenge = base64.urlsafe_b64encode(code_challenge).decode('utf-8')
        code_challenge = code_challenge.replace('=', '').replace('+', '-').replace('/', '_')
        
        return code_verifier, code_challenge
    
    def get_token(self):
        """Get access token using PKCE flow"""
        code_verifier, code_challenge = self.generate_pkce_pair()
        
        # Build authorization URL
        params = {
            'client_id': self.client_id,
            'response_type': 'code',
            'redirect_uri': 'http://localhost:8080/callback',
            'scope': f'api://{self.client_id}/.default',
            'code_challenge': code_challenge,
            'code_challenge_method': 'S256'
        }
        
        auth_url = f"https://login.microsoftonline.com/{self.tenant_id}/oauth2/v2.0/authorize"
        auth_url += '?' + urlencode(params)
        
        # Open browser for user authentication
        print(f"Opening browser for authentication...")
        webbrowser.open(auth_url)
        
        # Start local server to receive callback
        self.start_callback_server()
        
        # Exchange code for token
        token_url = f"https://login.microsoftonline.com/{self.tenant_id}/oauth2/v2.0/token"
        data = {
            'client_id': self.client_id,
            'code': self.auth_code,
            'redirect_uri': 'http://localhost:8080/callback',
            'grant_type': 'authorization_code',
            'code_verifier': code_verifier
        }
        
        response = requests.post(token_url, data=data)
        response.raise_for_status()
        
        return response.json()['access_token']
    
    def start_callback_server(self):
        """Start temporary server to receive OAuth callback"""
        class CallbackHandler(http.server.BaseHTTPRequestHandler):
            def do_GET(handler_self):
                from urllib.parse import urlparse, parse_qs
                query = urlparse(handler_self.path).query
                params = parse_qs(query)
                self.auth_code = params.get('code', [None])[0]
                
                handler_self.send_response(200)
                handler_self.send_header('Content-type', 'text/html')
                handler_self.end_headers()
                handler_self.wfile.write(b'<h1>Success! You can close this window.</h1>')
            
            def log_message(self, format, *args):
                pass
        
        server = http.server.HTTPServer(('localhost', 8080), CallbackHandler)
        server.handle_request()

# Usage
if __name__ == '__main__':
    oauth = OAuthPKCE(
        client_id='your-client-id',
        tenant_id='your-tenant-id'
    )
    
    token = oauth.get_token()
    print(f"Access Token: {token}")
    
    # Use token
    headers = {'Authorization': f'Bearer {token}'}
    response = requests.post('https://gateway-endpoint', headers=headers, json={...})
```

---

## 2. Device Code Flow

**Best for**: CLI tools, SSH sessions, devices without browsers

**No browser needed!** User enters code on another device.

### How It Works

```
1. Request device code
2. Display code to user
3. User visits URL and enters code
4. Poll for token
```

### Example: CLI Tool

```bash
#!/bin/bash
# device-login.sh - Get token using device code flow

CLIENT_ID=$(terraform output -raw entra_app_client_id)
TENANT_ID=$(terraform output -raw entra_tenant_id)

# Request device code
DEVICE_CODE_RESPONSE=$(curl -s -X POST \
  "https://login.microsoftonline.com/$TENANT_ID/oauth2/v2.0/devicecode" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=$CLIENT_ID" \
  -d "scope=api://${CLIENT_ID}/.default")

USER_CODE=$(echo "$DEVICE_CODE_RESPONSE" | jq -r '.user_code')
DEVICE_CODE=$(echo "$DEVICE_CODE_RESPONSE" | jq -r '.device_code')
VERIFICATION_URI=$(echo "$DEVICE_CODE_RESPONSE" | jq -r '.verification_uri')
EXPIRES_IN=$(echo "$DEVICE_CODE_RESPONSE" | jq -r '.expires_in')
INTERVAL=$(echo "$DEVICE_CODE_RESPONSE" | jq -r '.interval')

echo "============================================"
echo "To sign in, use a web browser to open:"
echo "$VERIFICATION_URI"
echo ""
echo "And enter the code: $USER_CODE"
echo "============================================"
echo ""
echo "Waiting for authentication..."

# Poll for token
MAX_ATTEMPTS=$((EXPIRES_IN / INTERVAL))
for i in $(seq 1 $MAX_ATTEMPTS); do
    sleep $INTERVAL
    
    TOKEN_RESPONSE=$(curl -s -X POST \
      "https://login.microsoftonline.com/$TENANT_ID/oauth2/v2.0/token" \
      -H "Content-Type: application/x-www-form-urlencoded" \
      -d "client_id=$CLIENT_ID" \
      -d "grant_type=urn:ietf:params:oauth:grant-type:device_code" \
      -d "device_code=$DEVICE_CODE")
    
    ERROR=$(echo "$TOKEN_RESPONSE" | jq -r '.error // empty')
    
    if [ "$ERROR" = "authorization_pending" ]; then
        echo "Waiting for user authentication..."
        continue
    elif [ -z "$ERROR" ]; then
        ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')
        echo ""
        echo "✅ Authentication successful!"
        echo "Access Token: $ACCESS_TOKEN"
        
        # Save token for later use
        echo "$ACCESS_TOKEN" > ~/.aws-lambda-mcp-token
        echo "Token saved to ~/.aws-lambda-mcp-token"
        exit 0
    else
        echo "Error: $ERROR"
        exit 1
    fi
done

echo "Authentication timed out"
exit 1
```

### Python Example

```python
import time
import requests

def device_code_flow(client_id, tenant_id):
    """Authenticate using device code flow"""
    
    # Request device code
    device_url = f"https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/devicecode"
    data = {
        'client_id': client_id,
        'scope': f'api://{client_id}/.default'
    }
    
    response = requests.post(device_url, data=data)
    response.raise_for_status()
    device_data = response.json()
    
    # Display instructions to user
    print("=" * 60)
    print(f"Visit: {device_data['verification_uri']}")
    print(f"Enter code: {device_data['user_code']}")
    print("=" * 60)
    print("Waiting for authentication...")
    
    # Poll for token
    token_url = f"https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/token"
    poll_data = {
        'client_id': client_id,
        'grant_type': 'urn:ietf:params:oauth:grant-type:device_code',
        'device_code': device_data['device_code']
    }
    
    expires_at = time.time() + device_data['expires_in']
    interval = device_data['interval']
    
    while time.time() < expires_at:
        time.sleep(interval)
        
        response = requests.post(token_url, data=poll_data)
        result = response.json()
        
        if 'error' not in result:
            print("\n✅ Authentication successful!")
            return result['access_token']
        
        if result['error'] != 'authorization_pending':
            raise Exception(f"Authentication failed: {result['error']}")
        
        print(".", end="", flush=True)
    
    raise Exception("Authentication timed out")

# Usage
if __name__ == '__main__':
    token = device_code_flow(
        client_id='your-client-id',
        tenant_id='your-tenant-id'
    )
    
    print(f"Token: {token}")
```

---

## 3. Single-Page Application (SPA)

**Best for**: Browser-based apps, JavaScript frontends

**No backend needed!** Pure frontend authentication.

### JavaScript Example

```javascript
// oauth-spa.js - Browser-based PKCE authentication

class EntraOAuth {
    constructor(clientId, tenantId) {
        this.clientId = clientId;
        this.tenantId = tenantId;
        this.redirectUri = window.location.origin;
    }
    
    // Generate PKCE parameters
    async generatePKCE() {
        const randomBytes = new Uint8Array(32);
        crypto.getRandomValues(randomBytes);
        const codeVerifier = btoa(String.fromCharCode(...randomBytes))
            .replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
        
        const encoder = new TextEncoder();
        const data = encoder.encode(codeVerifier);
        const hash = await crypto.subtle.digest('SHA-256', data);
        const codeChallenge = btoa(String.fromCharCode(...new Uint8Array(hash)))
            .replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
        
        return { codeVerifier, codeChallenge };
    }
    
    // Start authentication
    async login() {
        const { codeVerifier, codeChallenge } = await this.generatePKCE();
        
        // Store verifier for later
        sessionStorage.setItem('code_verifier', codeVerifier);
        
        // Build authorization URL
        const params = new URLSearchParams({
            client_id: this.clientId,
            response_type: 'code',
            redirect_uri: this.redirectUri,
            scope: `api://${this.clientId}/.default`,
            code_challenge: codeChallenge,
            code_challenge_method: 'S256'
        });
        
        const authUrl = `https://login.microsoftonline.com/${this.tenantId}/oauth2/v2.0/authorize?${params}`;
        window.location.href = authUrl;
    }
    
    // Handle callback
    async handleCallback() {
        const params = new URLSearchParams(window.location.search);
        const code = params.get('code');
        
        if (!code) {
            throw new Error('No authorization code found');
        }
        
        const codeVerifier = sessionStorage.getItem('code_verifier');
        sessionStorage.removeItem('code_verifier');
        
        // Exchange code for token
        const tokenUrl = `https://login.microsoftonline.com/${this.tenantId}/oauth2/v2.0/token`;
        const body = new URLSearchParams({
            client_id: this.clientId,
            code: code,
            redirect_uri: this.redirectUri,
            grant_type: 'authorization_code',
            code_verifier: codeVerifier
        });
        
        const response = await fetch(tokenUrl, {
            method: 'POST',
            headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
            body: body.toString()
        });
        
        const data = await response.json();
        return data.access_token;
    }
    
    // Call API with token
    async callGateway(token, action, parameters) {
        const response = await fetch('https://your-gateway-endpoint', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ action, parameters })
        });
        
        return response.json();
    }
}

// Usage
const oauth = new EntraOAuth('your-client-id', 'your-tenant-id');

// Login button
document.getElementById('login-btn').addEventListener('click', () => {
    oauth.login();
});

// On page load, check for callback
if (window.location.search.includes('code=')) {
    oauth.handleCallback().then(token => {
        console.log('Access Token:', token);
        
        // Use token
        oauth.callGateway(token, 'get_weather', { location: 'Sydney' })
            .then(result => console.log(result));
    });
}
```

---

## Comparison

| Flow | User Experience | Use Case | Browser Required | Secret Required |
|------|----------------|----------|------------------|-----------------|
| **PKCE** | Opens browser once | CLI tools, desktop apps | Yes (once) | ❌ No |
| **Device Code** | Visit URL, enter code | SSH, terminals, IoT | Yes (separate) | ❌ No |
| **SPA** | Native browser auth | Web apps, PWAs | Yes (native) | ❌ No |
| Client Credentials | None | Service-to-service | No | ✅ Yes |

## Recommendations

✅ **Use PKCE for**: Command-line tools, desktop applications  
✅ **Use Device Code for**: SSH sessions, terminals, devices without browsers  
✅ **Use SPA for**: Browser-based frontends, single-page apps  
❌ **Avoid Client Credentials for**: User-facing applications (requires hardcoded secret)

## Security Notes

- ✅ **No secrets in source code**: All flows are secret-less
- ✅ **User context**: Tokens represent actual user identity
- ✅ **Audit trail**: All authentications logged in Entra ID
- ✅ **Token refresh**: Refresh tokens can extend sessions
- ✅ **Conditional Access**: Entra ID policies apply
- ✅ **MFA support**: Enforced at authentication time

All three flows are **production-ready** and more secure than password-based flows!
