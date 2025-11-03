#!/bin/bash
set -e

echo "=== Refreshing OAuth Access Token ==="
echo ""

# Load from .env
if [ -f .env ]; then
  source .env
fi

# Check for required variables
if [ -z "$MCP_REFRESH_TOKEN" ] || [ -z "$MCP_CLIENT_ID" ] || [ -z "$MCP_TENANT_ID" ]; then
  echo "❌ Missing credentials. Run ./get-token.sh first."
  exit 1
fi

echo "Refreshing access token..."

# Request new token using refresh token
# Use client_id/.default scope (not api:// prefix)
TOKEN_RESPONSE=$(curl -s -X POST "https://login.microsoftonline.com/$MCP_TENANT_ID/oauth2/v2.0/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=$MCP_CLIENT_ID" \
  -d "grant_type=refresh_token" \
  -d "refresh_token=$MCP_REFRESH_TOKEN" \
  -d "scope=${MCP_CLIENT_ID}/.default offline_access")

# Check for errors
if echo "$TOKEN_RESPONSE" | jq -e '.error' > /dev/null 2>&1; then
  echo "❌ Error:"
  echo "$TOKEN_RESPONSE" | jq -r '.error_description // .error'
  echo ""
  echo "Refresh token expired. Run ./get-token.sh"
  exit 1
fi

# Extract tokens
NEW_ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')
NEW_REFRESH_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.refresh_token // empty')

# Keep old refresh token if not rotated
if [ -z "$NEW_REFRESH_TOKEN" ] || [ "$NEW_REFRESH_TOKEN" == "null" ]; then
  NEW_REFRESH_TOKEN="$MCP_REFRESH_TOKEN"
fi

if [ -z "$NEW_ACCESS_TOKEN" ] || [ "$NEW_ACCESS_TOKEN" == "null" ]; then
  echo "❌ Failed to refresh"
  exit 1
fi

echo "✅ Success!"
echo ""

# Update .env
cat > .env << EOF
# Generated: $(date)
export MCP_ACCESS_TOKEN="$NEW_ACCESS_TOKEN"
export MCP_REFRESH_TOKEN="$NEW_REFRESH_TOKEN"
export MCP_CLIENT_ID="$MCP_CLIENT_ID"
export MCP_TENANT_ID="$MCP_TENANT_ID"
EOF

echo "Token saved to .env"
echo ""
echo "=================================================="
echo "🚀 Launching MCP Inspector"
echo "=================================================="
echo ""

# Get gateway URL
GATEWAY_URL=$(terraform output -raw bedrock_gateway_url 2>/dev/null || echo "")

if [ -z "$GATEWAY_URL" ] || [ "$GATEWAY_URL" == "null" ]; then
  echo "⚠️  Gateway not deployed"
  exit 0
fi

echo "Gateway: $GATEWAY_URL"
echo ""

# Launch inspector
npx @modelcontextprotocol/inspector "$GATEWAY_URL" \
  --authorization-token "Bearer $NEW_ACCESS_TOKEN"
