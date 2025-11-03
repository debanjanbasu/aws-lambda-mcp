#!/bin/bash
set -e

echo "=== AWS Lambda MCP Gateway - OAuth Token Generator ==="
echo ""

# Get Terraform outputs
CLIENT_ID=$(terraform output -raw entra_app_client_id 2>/dev/null)
TENANT_ID=$(terraform output -raw entra_tenant_id 2>/dev/null)
APP_NAME=$(terraform output -raw entra_app_name 2>/dev/null)
AWS_REGION=$(terraform output -raw aws_region 2>/dev/null || echo "ap-southeast-2")

if [ -z "$CLIENT_ID" ] || [ -z "$TENANT_ID" ]; then
  echo "❌ Error: Could not read Terraform outputs"
  echo "Run 'terraform apply' first"
  exit 1
fi

echo "Client ID: $CLIENT_ID"
echo "Tenant ID: $TENANT_ID"
echo "App Name: $APP_NAME"
echo ""

# Generate PKCE parameters (RFC 7636)
echo "Generating PKCE parameters..."
CODE_VERIFIER=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-43)
CODE_CHALLENGE=$(echo -n "$CODE_VERIFIER" | openssl dgst -sha256 -binary | base64 | tr '+/' '-_' | tr -d '=')

REDIRECT_URI="http://localhost:6274/callback/"

# Request token with the correct scope for our API
SCOPE="api://${CLIENT_ID}/access_as_user openid profile email offline_access"

# Build authorization URL
AUTH_URL="https://login.microsoftonline.com/$TENANT_ID/oauth2/v2.0/authorize"
AUTH_URL="${AUTH_URL}?client_id=${CLIENT_ID}"
AUTH_URL="${AUTH_URL}&response_type=code"
AUTH_URL="${AUTH_URL}&redirect_uri=$(printf %s "$REDIRECT_URI" | jq -sRr @uri)"
AUTH_URL="${AUTH_URL}&response_mode=query"
AUTH_URL="${AUTH_URL}&scope=$(printf %s "$SCOPE" | jq -sRr @uri)"
AUTH_URL="${AUTH_URL}&code_challenge=${CODE_CHALLENGE}"
AUTH_URL="${AUTH_URL}&code_challenge_method=S256"

echo ""
echo "=================================================="
echo "🔐 STEP 1: Authenticate in Browser"
echo "=================================================="
echo ""

# Open browser
if command -v open &> /dev/null; then
  open "$AUTH_URL"
elif command -v xdg-open &> /dev/null; then
  xdg-open "$AUTH_URL"
else
  echo "Open this URL:"
  echo "$AUTH_URL"
fi

echo "After authentication, paste the redirect URL..."
echo ""
read -p "Redirect URL: " REDIRECT_RESPONSE

# Extract authorization code
if [[ "$REDIRECT_RESPONSE" =~ code=([^&]+) ]]; then
  AUTH_CODE="${BASH_REMATCH[1]}"
else
  echo "❌ Could not extract authorization code"
  exit 1
fi

echo ""
echo "=================================================="
echo "🔐 STEP 2: Exchange Code for Token"
echo "=================================================="
echo ""

# Exchange code for token using PKCE
TOKEN_RESPONSE=$(curl -s -X POST "https://login.microsoftonline.com/$TENANT_ID/oauth2/v2.0/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=$CLIENT_ID" \
  -d "grant_type=authorization_code" \
  -d "code=$AUTH_CODE" \
  -d "redirect_uri=$REDIRECT_URI" \
  -d "code_verifier=$CODE_VERIFIER")

# Check for errors
if echo "$TOKEN_RESPONSE" | jq -e '.error' > /dev/null 2>&1; then
  echo "❌ Error:"
  echo "$TOKEN_RESPONSE" | jq -r '.error_description // .error'
  exit 1
fi

# Extract tokens
ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')
REFRESH_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.refresh_token // empty')

if [ -z "$ACCESS_TOKEN" ] || [ "$ACCESS_TOKEN" == "null" ]; then
  echo "❌ Failed to get token"
  echo "$TOKEN_RESPONSE"
  exit 1
fi

echo "✅ Success!"
echo ""

# Save tokens
cat > .env << EOF
# Generated: $(date)
export MCP_ACCESS_TOKEN="$ACCESS_TOKEN"
export MCP_REFRESH_TOKEN="$REFRESH_TOKEN"
export MCP_CLIENT_ID="$CLIENT_ID"
export MCP_TENANT_ID="$TENANT_ID"
EOF

echo "Tokens saved to .env"
echo ""
echo "=================================================="
echo "🚀 Launching MCP Inspector"
echo "=================================================="
echo ""

# Get gateway URL for MCP Inspector
GATEWAY_URL=$(terraform output -raw bedrock_gateway_url 2>/dev/null || echo "")

if [ -z "$GATEWAY_URL" ] || [ "$GATEWAY_URL" == "null" ]; then
  echo "⚠️  Gateway not yet deployed"
  echo ""
  echo "Tokens saved to .env. To test manually after deployment:"
  echo "  source .env"
  echo "  npx @modelcontextprotocol/inspector <gateway-url> --authorization-token \"Bearer \$MCP_ACCESS_TOKEN\""
  exit 0
fi

echo "Gateway URL: $GATEWAY_URL"

echo "Gateway URL: $GATEWAY_URL"
echo ""

# Export for easy reuse
export MCP_ACCESS_TOKEN="$ACCESS_TOKEN"
export MCP_GATEWAY_URL="$GATEWAY_URL"

# Launch MCP Inspector
echo "Launching MCP Inspector..."
echo ""
npx @modelcontextprotocol/inspector "$GATEWAY_URL" \
  --authorization-token "Bearer $ACCESS_TOKEN"
