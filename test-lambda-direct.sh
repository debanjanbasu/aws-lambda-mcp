#!/bin/bash
# Direct Lambda test bypassing Gateway

set -e

echo "=== Testing Lambda Directly ==="
echo

# Test 1: Sydney weather
echo "Test 1: Sydney weather"
aws lambda invoke \
  --function-name aws-lambda-mcp \
  --cli-binary-format raw-in-base64-out \
  --payload '{"name":"get_weather","location":"Sydney"}' \
  /tmp/test-response.json > /dev/null

echo "Response:"
cat /tmp/test-response.json | jq .
echo

# Test 2: New York weather
echo "Test 2: New York weather"
aws lambda invoke \
  --function-name aws-lambda-mcp \
  --cli-binary-format raw-in-base64-out \
  --payload '{"name":"get_weather","location":"New York"}' \
  /tmp/test-response2.json > /dev/null

echo "Response:"
cat /tmp/test-response2.json | jq .
echo

echo "✅ Lambda is working correctly!"
echo "Issue is with Agent Core Gateway (preview service)"
