# AI Assistant Instructions for AWS Lambda MCP Infrastructure

## Overview

This document provides guidelines for AI assistants working on the infrastructure components of the AWS Lambda MCP project. Focus on Terraform, AWS resources, and deployment automation while maintaining security, scalability, and cost-efficiency.

## Table of Contents

- [Overview](#overview)
- [Key Infrastructure Files](#key-infrastructure-files)
- [Terraform Guidelines](#terraform-guidelines)
- [AWS Resource Best Practices](#aws-resource-best-practices)
- [Security Considerations](#security-considerations)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)

## Key Infrastructure Files

- `main.tf` - Main Terraform resources (Lambda, API Gateway, Bedrock)
- `variables.tf` - Input variables with defaults and validation
- `outputs.tf` - Output values for other systems
- `locals.tf` - Computed values and constants
- `providers.tf` - AWS and Azure provider configurations
- `backend.tf` - S3 backend configuration
- `Makefile` - Infrastructure-specific commands

## Terraform Guidelines

- **Modular Structure**: Use separate files for logical groupings (e.g., variables.tf, outputs.tf)
- **Resource Naming**: Follow consistent naming with `local.project_name_with_suffix`
- **State Management**: Use S3 backend with native locking; never commit state files
- **Variables**: Use `variables.tf` for all inputs; provide sensible defaults
- **Outputs**: Export necessary values (e.g., URLs, ARNs) for cross-system integration
- **Locals**: Use for computed values, constants, and reusable expressions
- **Validation**: Add variable validation rules where appropriate
- **Dependencies**: Use `depends_on` sparingly; prefer implicit dependencies
- **Formatting**: Run `terraform fmt` before committing

## AWS Resource Best Practices

- **Lambda**: Use ARM64 architecture, set appropriate memory/timeout, enable CloudWatch logs
- **API Gateway**: Use regional deployment, enable logging, configure CORS properly
- **IAM**: Follow least privilege; use managed policies where possible
- **S3**: Enable versioning, encryption, and access logging for state buckets
- **CloudWatch**: Set retention policies (90 days default), use structured logging
- **Cost Optimization**: Use free tier eligible services, monitor usage

## Security Considerations

- **Secrets Management**: Never hardcode secrets; use environment variables or AWS Secrets Manager
- **IAM Roles**: Restrict permissions to minimum required; use condition keys
- **Network Security**: Use VPC endpoints where possible; avoid public IPs
- **Logging**: Enable CloudTrail, avoid logging sensitive data
- **Compliance**: Follow AWS security best practices; regular audits

## Troubleshooting

- **Backend Issues**: Ensure `backend.config` exists; run `make setup-backend` if missing
- **Provider Auth**: Verify AWS/Azure credentials with `make login`
- **State Locks**: Use `terraform force-unlock` only as last resort
- **Resource Conflicts**: Check for naming collisions; use unique suffixes
- **Plan Failures**: Validate syntax with `terraform validate`; check variable values

## Contributing

- Run `terraform validate` and `terraform fmt` before changes
- Test with `make tf-plan` to verify no breaking changes
- Document new variables/outputs in comments
- For complex changes, create separate PRs for infrastructure updates</content>
<parameter name="filePath">iac/AGENTS.md