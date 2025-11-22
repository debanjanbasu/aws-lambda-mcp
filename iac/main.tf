# Amazon Bedrock AgentCore Gateway Infrastructure
# Main infrastructure resources for AWS Lambda and Amazon Bedrock AgentCore integration

# Create zip file from Lambda binary
# Use a conditional expression to avoid evaluating filemd5 when the file doesn't exist
data "archive_file" "lambda_zip" {
  type        = "zip"
  source_file = local.lambda_binary_path
  output_path = "${path.module}/.terraform/lambda.zip"
}

# Lambda Function
resource "aws_lambda_function" "bedrock_agent_gateway" {
  function_name = local.project_name_with_suffix
  role          = aws_iam_role.lambda_execution.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["arm64"]

  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256

  memory_size = var.lambda_memory_size
  timeout     = var.lambda_timeout

  # X-Ray tracing disabled to reduce costs (can enable if needed)
  # tracing_config {
  #   mode = "Active"
  # }

  environment {
    variables = merge(
      {
        RUST_LOG = var.rust_log_level
      },
      var.additional_env_vars
    )
  }

  depends_on = [
    aws_iam_role_policy_attachment.lambda_basic,
    aws_cloudwatch_log_group.lambda_logs,
  ]
}

# CloudWatch Log Group
resource "aws_cloudwatch_log_group" "lambda_logs" {
  name              = "/aws/lambda/${local.project_name_with_suffix}"
  retention_in_days = var.log_retention_days

  # KMS encryption disabled to reduce costs (free tier uses SSE)
  # kms_key_id = var.cloudwatch_kms_key_arn
}

# IAM Role for Lambda
resource "aws_iam_role" "lambda_execution" {
  name               = "${local.project_name_with_suffix}-lambda-role"
  assume_role_policy = data.aws_iam_policy_document.lambda_assume_role.json
}

data "aws_iam_policy_document" "lambda_assume_role" {
  statement {
    effect = "Allow"

    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }

    actions = ["sts:AssumeRole"]
  }
}

# Gateway assume role policy - allow Bedrock AgentCore service
# IMPORTANT: Both service principals are required for Amazon Bedrock AgentCore Gateway to work:
# - bedrock.amazonaws.com: Legacy service principal (may be used by some Gateway operations)
# - bedrock-agentcore.amazonaws.com: AgentCore-specific service principal for Lambda invocation
# Without both, you'll get "Access denied while invoking Lambda function" errors even if
# the Lambda resource policy and Gateway role policy are correctly configured.
data "aws_iam_policy_document" "gateway_assume_role" {
  statement {
    effect = "Allow"

    principals {
      type        = "Service"
      identifiers = ["bedrock.amazonaws.com", "bedrock-agentcore.amazonaws.com"]
    }

    actions = ["sts:AssumeRole"]
  }
}

# Basic Lambda Execution Policy
resource "aws_iam_role_policy_attachment" "lambda_basic" {
  role       = aws_iam_role.lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

# X-Ray Tracing Policy (disabled to reduce costs)
# resource "aws_iam_role_policy_attachment" "lambda_xray" {
#   role       = aws_iam_role.lambda_execution.name
#   policy_arn = "arn:aws:iam::aws:policy/AWSXRayDaemonWriteAccess"
# }

# IAM Role for Amazon Bedrock AgentCore Gateway
resource "aws_iam_role" "gateway_role" {
  name = "${local.project_name_with_suffix}-gateway-role"

  assume_role_policy = data.aws_iam_policy_document.gateway_assume_role.json

  tags = var.common_tags
}

# Policy allowing Amazon Bedrock AgentCore Gateway to invoke Lambda
resource "aws_iam_role_policy" "gateway_lambda_invoke" {
  name = "${local.project_name_with_suffix}-gateway-lambda-invoke"
  role = aws_iam_role.gateway_role.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = [
        "lambda:InvokeFunction"
      ]
      Resource = aws_lambda_function.bedrock_agent_gateway.arn
    }]
  })
}

# Amazon Bedrock AgentCore Gateway with JWT authorization
# Implements Model Context Protocol (MCP) server with semantic tool search
resource "aws_bedrockagentcore_gateway" "main" {
  name            = local.project_name_with_suffix
  protocol_type   = "MCP"
  role_arn        = aws_iam_role.gateway_role.arn
  authorizer_type = "CUSTOM_JWT"

  # Protocol configuration for MCP
  # SEMANTIC search type enables intelligent tool selection based on:
  # - Natural language query understanding
  # - Tool descriptions and parameter matching
  # - Context-aware tool recommendations
  # See: https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/gateway-using-mcp-semantic-search.html
  protocol_configuration {
    mcp {
      instructions = "Gateway for handling MCP requests"
      search_type  = "SEMANTIC"
    }
  }

  # Authorizer configuration for JWT validation
  # Uses OIDC discovery URL to validate tokens against Entra ID
  authorizer_configuration {
    custom_jwt_authorizer {
      discovery_url = local.entra_discovery_url
      allowed_audience = [
        "api://${azuread_application.agentcore_app.client_id}",
        azuread_application.agentcore_app.client_id
      ]
    }
  }

  # Exception level for error logging
  # Controls the verbosity of error messages returned by the Gateway:
  # - DEBUG: Most verbose - detailed context and debugging information
  # - INFO:  Informational messages about Gateway operations
  # - WARN:  Warning messages about potential issues
  # - ERROR: Only error messages
  # - null:  Minimal error information (default for security)
  #
  # Security consideration: Higher verbosity levels may expose sensitive information
  # in error responses. Use DEBUG/INFO only for troubleshooting, not production.
  #
  # Reference: https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/bedrockagentcore_gateway#exception_level-1
  exception_level = var.gateway_exception_level

  tags = var.common_tags
}

# Amazon Bedrock AgentCore Gateway Target (Lambda)
# Tool schemas hardcoded for the weather tool
resource "aws_bedrockagentcore_gateway_target" "lambda" {
  name               = "${local.project_name_with_suffix}-target"
  gateway_identifier = aws_bedrockagentcore_gateway.main.gateway_id
  description        = "Lambda target with MCP weather tool"

  target_configuration {
    mcp {
      lambda {
        lambda_arn = aws_lambda_function.bedrock_agent_gateway.arn

        tool_schema {
          inline_payload {
            name        = "get_weather"
            description = "Get current weather information for a specified location. Returns temperature (automatically converted to Celsius or Fahrenheit based on the country), WMO weather code, and wind speed in km/h. Supports city names, addresses, or place names worldwide."

            input_schema {
              type        = "object"
              description = "Request for weather information"

              property {
                name        = "location"
                type        = "string"
                description = "Location name (city, address, or place)"
                required    = true
              }
            }

            output_schema {
              type        = "object"
              description = "Response containing weather information"

              property {
                name        = "location"
                type        = "string"
                description = "Location name"
                required    = true
              }

              property {
                name        = "temperature"
                type        = "number"
                description = "Temperature value"
                required    = true
              }

              property {
                name        = "temperature_unit"
                type        = "string"
                description = "The unit of temperature (Celsius or Fahrenheit)"
                required    = true
              }

              property {
                name        = "weather_code"
                type        = "integer"
                description = "WMO weather code"
                required    = true
              }

              property {
                name        = "wind_speed"
                type        = "number"
                description = "Wind speed in km/h"
                required    = true
              }
            }
          }
        }
      }
    }
  }

  credential_provider_configuration {
    gateway_iam_role {}
  }
}

# Lambda permission for Amazon Bedrock AgentCore Gateway to invoke
resource "aws_lambda_permission" "agentcore_gateway_invoke" {
  statement_id  = "AllowAgentCoreGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.bedrock_agent_gateway.function_name
  principal     = "bedrock.amazonaws.com"
  source_arn    = aws_bedrockagentcore_gateway.main.gateway_arn
}



# Lambda Alias (removed - not needed, adds version management overhead)
# Use $LATEST directly to avoid versioning costs
# resource "aws_lambda_alias" "live" {
#   name             = var.lambda_alias_name
#   function_name    = aws_lambda_function.bedrock_agent_gateway.function_name
#   function_version = var.lambda_alias_version
# }

# CloudWatch Alarms removed entirely to minimize costs
# Standard CloudWatch alarms cost $0.10/alarm/month (NOT in free tier)
# Use CloudWatch Insights queries instead (FREE):
#   - Error rate: fields @timestamp | filter level = "ERROR" | stats count()
#   - Duration: fields @timestamp, @duration | stats avg(@duration), max(@duration)
#   - Invocations: Open Lambda console → Monitor tab (free metrics)
#
# If you need alarms for production, uncomment and set enable_alarms = true:
#
# resource "aws_cloudwatch_metric_alarm" "lambda_errors" {
#   alarm_name          = "${var.lambda_function_name}-errors"
#   comparison_operator = "GreaterThanThreshold"
#   evaluation_periods  = 2
#   metric_name         = "Errors"
#   namespace           = "AWS/Lambda"
#   period              = 300
#   statistic           = "Sum"
#   threshold           = 5
#   alarm_description   = "Lambda function error rate"
#   treat_missing_data  = "notBreaching"
#
#   dimensions = {
#     FunctionName = aws_lambda_function.bedrock_agent_gateway.function_name
#   }
#
#   alarm_actions = [] # Add SNS topic ARN if needed
# }
