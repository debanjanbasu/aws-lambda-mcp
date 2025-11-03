# Terraform Configuration Fixes

Fixed schema changes for AWS Provider 6.19.0 (auto-upgraded from 5.75).

## Changes Made

### 1. Bedrock Gateway Resource Schema

**Before (incorrect):**
```hcl
resource "aws_bedrockagentcore_gateway" "main" {
  gateway_name = var.gateway_name  # Wrong attribute
  protocol_type = "LAMBDA"          # Wrong enum
  authorizer_type = "OIDC"          # Wrong enum
  # Missing role_arn
  
  authorizer_configuration {
    openid_connect {                # Wrong block
      issuer_url = "..."
      audience = [...]
    }
  }
}
```

**After (correct):**
```hcl
resource "aws_bedrockagentcore_gateway" "main" {
  name = var.gateway_name          # Correct attribute
  protocol_type = "MCP"            # Correct enum
  authorizer_type = "CUSTOM_JWT"   # Correct enum
  role_arn = aws_iam_role.bedrock_gateway.arn  # Required
  
  authorizer_configuration {
    custom_jwt_authorizer {        # Correct block
      discovery_url = "https://login.microsoftonline.com/{tenant}/v2.0/.well-known/openid-configuration"
      allowed_audience = [azuread_application.bedrock_gateway.client_id]
    }
  }
}
```

### 2. Added IAM Role for Gateway

```hcl
resource "aws_iam_role" "bedrock_gateway" {
  name = "${var.gateway_name}-role"
  
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Principal = {
        Service = "bedrock.amazonaws.com"
      }
      Action = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy" "bedrock_gateway_lambda" {
  name = "${var.gateway_name}-lambda-invoke"
  role = aws_iam_role.bedrock_gateway.id
  
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = ["lambda:InvokeFunction"]
      Resource = aws_lambda_function.bedrock_agent_gateway.arn
    }]
  })
}
```

### 3. Fixed Gateway Target Reference

**Before:**
```hcl
gateway_identifier = aws_bedrockagentcore_gateway.main.id  # Wrong
```

**After:**
```hcl
gateway_identifier = aws_bedrockagentcore_gateway.main.gateway_id  # Correct
```

### 4. Fixed Outputs

**Before:**
```hcl
value = aws_bedrockagentcore_gateway.main.gateway_name  # No such attribute
```

**After:**
```hcl
# Available attributes
value = aws_bedrockagentcore_gateway.main.name          # Gateway name
value = aws_bedrockagentcore_gateway.main.gateway_id    # Gateway ID
value = aws_bedrockagentcore_gateway.main.gateway_arn   # Gateway ARN
value = aws_bedrockagentcore_gateway.main.gateway_url   # Gateway URL
```

## Key Schema Changes

| Attribute | Old | New |
|-----------|-----|-----|
| Gateway name | `gateway_name` | `name` |
| Protocol | `LAMBDA` | `MCP` |
| Authorizer | `OIDC` | `CUSTOM_JWT` |
| Auth block | `openid_connect` | `custom_jwt_authorizer` |
| Issuer config | `issuer_url` + `audience` | `discovery_url` + `allowed_audience` |
| Gateway ref | `.id` | `.gateway_id` |
| Role | Not required | **Required** `role_arn` |

## Why These Changes?

AWS Provider 6.x introduced breaking changes:
- **MCP protocol**: Model Context Protocol (not Lambda-specific)
- **CUSTOM_JWT**: More flexible JWT validation
- **Discovery URL**: Uses OpenID Connect discovery (best practice)
- **IAM Role**: Explicit role required for service-to-service calls

## Testing

```bash
# Format
terraform fmt

# Validate
terraform validate

# Plan (requires AWS + Azure credentials)
terraform plan

# Apply
terraform apply
```

## Benefits of Discovery URL

Using `discovery_url` instead of manual issuer/JWKS:
✅ Automatic key rotation support
✅ Fetches issuer, JWKS URI, algorithms automatically
✅ Standard OpenID Connect discovery
✅ Future-proof against Entra ID changes

---

**Fixed**: 2025-11-03  
**AWS Provider**: 6.19.0
