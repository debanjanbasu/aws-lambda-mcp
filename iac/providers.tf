# Terraform and Provider Configuration
# This file defines required Terraform version and provider configurations

terraform {
  required_version = ">= 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = ">= 6.0"
    }
    archive = {
      source  = "hashicorp/archive"
      version = ">= 2.6"
    }
    azuread = {
      source  = "hashicorp/azuread"
      version = ">= 3.7"
    }
    random = {
      source  = "hashicorp/random"
      version = ">= 3.0"
    }
  }
}

# AWS Provider Configuration
provider "aws" {
  region = var.aws_region

  default_tags {
    tags = var.common_tags
  }
}

# Entra ID Provider Configuration
provider "azuread" {
  # Configure via environment variables:
  # export ARM_TENANT_ID="your-tenant-id"
  # export ARM_CLIENT_ID="your-client-id"
  # export ARM_CLIENT_SECRET="your-client-secret"
  #
  # Or use Azure CLI authentication:
  # az login
}
