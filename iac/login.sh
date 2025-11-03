#!/bin/bash
# Quick login script for AWS and Azure

set -e

echo "=== AWS Lambda MCP - Login Helper ==="
echo ""

# Check AWS CLI
if ! command -v aws &> /dev/null; then
    echo "❌ AWS CLI not found. Install: https://aws.amazon.com/cli/"
    exit 1
fi

# Check Azure CLI
if ! command -v az &> /dev/null; then
    echo "❌ Azure CLI not found. Install: https://aka.ms/azure-cli"
    exit 1
fi

echo "1️⃣  Logging in to AWS..."
echo ""

# AWS Login
if aws sts get-caller-identity &> /dev/null; then
    echo "✅ Already logged in to AWS:"
    aws sts get-caller-identity --query 'Account' --output text
else
    echo "Please login to AWS:"
    if aws configure list | grep -q sso_start_url; then
        aws sso login
    else
        echo "Run: aws configure"
        echo "Then try again."
        exit 1
    fi
fi

echo ""
echo "2️⃣  Logging in to Azure..."
echo ""

# Azure Login
if az account show &> /dev/null; then
    echo "✅ Already logged in to Azure:"
    az account show --query 'name' -o tsv
else
    echo "Opening browser for Azure login..."
    az login
fi

echo ""
echo "✅ Login complete!"
echo ""
echo "Next steps:"
echo "  cd iac"
echo "  terraform init"
echo "  terraform apply"
echo ""
