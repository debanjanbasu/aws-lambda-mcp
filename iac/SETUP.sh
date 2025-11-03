#!/bin/bash
# SETUP.sh - Login to AWS and Azure for Terraform deployment

set -e

echo "=========================================="
echo "AWS Lambda MCP - Local Setup"
echo "=========================================="
echo ""

# Check prerequisites
echo "Checking prerequisites..."
command -v aws >/dev/null 2>&1 || { echo "❌ AWS CLI not found. Install: brew install awscli"; exit 1; }
command -v az >/dev/null 2>&1 || { echo "❌ Azure CLI not found. Install: brew install azure-cli"; exit 1; }
command -v terraform >/dev/null 2>&1 || { echo "❌ Terraform not found. Install: brew install terraform"; exit 1; }
echo "✅ All tools installed"
echo ""

# AWS Login
echo "=========================================="
echo "1. AWS Login"
echo "=========================================="
echo ""
echo "Choose AWS authentication method:"
echo "  1) AWS SSO (recommended)"
echo "  2) AWS IAM credentials"
echo "  3) Skip (already logged in)"
read -p "Select [1-3]: " aws_choice

case $aws_choice in
  1)
    read -p "Enter SSO start URL: " sso_url
    read -p "Enter SSO region [ap-southeast-2]: " sso_region
    sso_region=${sso_region:-ap-southeast-2}
    
    echo "Logging in to AWS SSO..."
    aws sso login --sso-start-url "$sso_url" --sso-region "$sso_region"
    
    echo ""
    echo "Testing AWS access..."
    aws sts get-caller-identity
    ;;
  2)
    echo "Configure AWS credentials..."
    aws configure
    
    echo ""
    echo "Testing AWS access..."
    aws sts get-caller-identity
    ;;
  3)
    echo "Verifying existing AWS session..."
    if aws sts get-caller-identity >/dev/null 2>&1; then
      echo "✅ AWS session valid"
      aws sts get-caller-identity
    else
      echo "❌ No valid AWS session found"
      exit 1
    fi
    ;;
esac

echo ""
echo "✅ AWS authentication complete"
echo ""

# Azure Login
echo "=========================================="
echo "2. Azure Login"
echo "=========================================="
echo ""
echo "Choose Azure authentication method:"
echo "  1) Azure CLI (interactive)"
echo "  2) Service Principal (CI/CD)"
echo "  3) Skip (already logged in)"
read -p "Select [1-3]: " azure_choice

case $azure_choice in
  1)
    echo "Opening browser for Azure login..."
    az login
    
    echo ""
    echo "Select subscription (if you have multiple):"
    az account list --output table
    echo ""
    read -p "Enter subscription ID (or press Enter to use default): " subscription_id
    
    if [ -n "$subscription_id" ]; then
      az account set --subscription "$subscription_id"
    fi
    
    echo ""
    echo "Testing Azure access..."
    az account show
    
    echo ""
    echo "Creating service principal for Terraform..."
    echo "This allows Terraform to create Entra ID resources."
    echo ""
    
    SP_NAME="terraform-aws-lambda-mcp-$(date +%s)"
    SUBSCRIPTION_ID=$(az account show --query id -o tsv)
    
    echo "Creating service principal: $SP_NAME"
    SP_OUTPUT=$(az ad sp create-for-rbac \
      --name "$SP_NAME" \
      --role Contributor \
      --scopes "/subscriptions/$SUBSCRIPTION_ID" \
      --sdk-auth)
    
    ARM_CLIENT_ID=$(echo "$SP_OUTPUT" | jq -r '.clientId')
    ARM_CLIENT_SECRET=$(echo "$SP_OUTPUT" | jq -r '.clientSecret')
    ARM_TENANT_ID=$(echo "$SP_OUTPUT" | jq -r '.tenantId')
    ARM_SUBSCRIPTION_ID=$(echo "$SP_OUTPUT" | jq -r '.subscriptionId')
    
    echo ""
    echo "✅ Service principal created!"
    echo ""
    echo "Add these to your shell profile (~/.zshrc or ~/.bashrc):"
    echo ""
    echo "export ARM_TENANT_ID=\"$ARM_TENANT_ID\""
    echo "export ARM_CLIENT_ID=\"$ARM_CLIENT_ID\""
    echo "export ARM_CLIENT_SECRET=\"$ARM_CLIENT_SECRET\""
    echo "export ARM_SUBSCRIPTION_ID=\"$ARM_SUBSCRIPTION_ID\""
    echo ""
    
    # Export for current session
    export ARM_TENANT_ID="$ARM_TENANT_ID"
    export ARM_CLIENT_ID="$ARM_CLIENT_ID"
    export ARM_CLIENT_SECRET="$ARM_CLIENT_SECRET"
    export ARM_SUBSCRIPTION_ID="$ARM_SUBSCRIPTION_ID"
    
    echo "Environment variables set for current session."
    ;;
  2)
    echo "Enter service principal credentials:"
    read -p "ARM_TENANT_ID: " ARM_TENANT_ID
    read -p "ARM_CLIENT_ID: " ARM_CLIENT_ID
    read -sp "ARM_CLIENT_SECRET: " ARM_CLIENT_SECRET
    echo ""
    read -p "ARM_SUBSCRIPTION_ID: " ARM_SUBSCRIPTION_ID
    
    export ARM_TENANT_ID="$ARM_TENANT_ID"
    export ARM_CLIENT_ID="$ARM_CLIENT_ID"
    export ARM_CLIENT_SECRET="$ARM_CLIENT_SECRET"
    export ARM_SUBSCRIPTION_ID="$ARM_SUBSCRIPTION_ID"
    
    echo "Testing service principal..."
    az login --service-principal \
      -u "$ARM_CLIENT_ID" \
      -p "$ARM_CLIENT_SECRET" \
      --tenant "$ARM_TENANT_ID"
    
    az account set --subscription "$ARM_SUBSCRIPTION_ID"
    az account show
    ;;
  3)
    echo "Verifying existing Azure session..."
    if az account show >/dev/null 2>&1; then
      echo "✅ Azure session valid"
      az account show
      
      echo ""
      echo "Checking for ARM environment variables..."
      if [ -z "$ARM_TENANT_ID" ] || [ -z "$ARM_CLIENT_ID" ] || [ -z "$ARM_CLIENT_SECRET" ]; then
        echo "⚠️  ARM_* environment variables not set"
        echo "Terraform needs these for Entra ID operations."
        echo "Do you want to create a service principal? [y/N]"
        read -p "> " create_sp
        
        if [ "$create_sp" = "y" ]; then
          SP_NAME="terraform-aws-lambda-mcp-$(date +%s)"
          SUBSCRIPTION_ID=$(az account show --query id -o tsv)
          
          SP_OUTPUT=$(az ad sp create-for-rbac \
            --name "$SP_NAME" \
            --role Contributor \
            --scopes "/subscriptions/$SUBSCRIPTION_ID" \
            --sdk-auth)
          
          ARM_CLIENT_ID=$(echo "$SP_OUTPUT" | jq -r '.clientId')
          ARM_CLIENT_SECRET=$(echo "$SP_OUTPUT" | jq -r '.clientSecret')
          ARM_TENANT_ID=$(echo "$SP_OUTPUT" | jq -r '.tenantId')
          ARM_SUBSCRIPTION_ID=$(echo "$SP_OUTPUT" | jq -r '.subscriptionId')
          
          export ARM_TENANT_ID="$ARM_TENANT_ID"
          export ARM_CLIENT_ID="$ARM_CLIENT_ID"
          export ARM_CLIENT_SECRET="$ARM_CLIENT_SECRET"
          export ARM_SUBSCRIPTION_ID="$ARM_SUBSCRIPTION_ID"
          
          echo ""
          echo "✅ Service principal created and exported!"
          echo ""
          echo "Add these to ~/.zshrc or ~/.bashrc:"
          echo ""
          echo "export ARM_TENANT_ID=\"$ARM_TENANT_ID\""
          echo "export ARM_CLIENT_ID=\"$ARM_CLIENT_ID\""
          echo "export ARM_CLIENT_SECRET=\"$ARM_CLIENT_SECRET\""
          echo "export ARM_SUBSCRIPTION_ID=\"$ARM_SUBSCRIPTION_ID\""
        fi
      else
        echo "✅ ARM environment variables are set"
      fi
    else
      echo "❌ No valid Azure session found"
      exit 1
    fi
    ;;
esac

echo ""
echo "✅ Azure authentication complete"
echo ""

# Summary
echo "=========================================="
echo "3. Summary"
echo "=========================================="
echo ""
echo "AWS Account:"
aws sts get-caller-identity --query "Arn" --output text
echo ""
echo "Azure Account:"
az account show --query "user.name" --output tsv
echo ""
echo "Azure Subscription:"
az account show --query "name" --output tsv
echo ""

if [ -n "$ARM_TENANT_ID" ]; then
  echo "Azure Tenant ID: $ARM_TENANT_ID"
  echo ""
fi

echo "=========================================="
echo "4. Next Steps"
echo "=========================================="
echo ""
echo "1. Build the Lambda binary:"
echo "   cd .. && make release"
echo ""
echo "2. Configure Terraform:"
echo "   cd iac"
echo "   cp terraform.tfvars.example terraform.tfvars"
echo "   # Edit terraform.tfvars with your settings"
echo ""
echo "3. Deploy:"
echo "   terraform init"
echo "   terraform plan"
echo "   terraform apply"
echo ""
echo "✅ Setup complete!"
