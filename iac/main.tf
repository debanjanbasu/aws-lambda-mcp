# AWS Bedrock AgentCore Gateway Infrastructure
# Main infrastructure resources for AWS Lambda and AWS Bedrock AgentCore integration

# Create zip file from Lambda binary
data "archive_file" "lambda_zip" {
  type        = "zip"
  source_file = local.lambda_binary_path
  output_path = "${path.module}/.terraform/lambda-${filemd5(local.lambda_binary_path)}.zip"
}

# Lambda Function
resource "aws_lambda_function" "bedrock_agent_gateway" {
  function_name = var.project_name
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
  name              = "/aws/lambda/${var.project_name}"
  retention_in_days = var.log_retention_days

  # KMS encryption disabled to reduce costs (free tier uses SSE)
  # kms_key_id = var.cloudwatch_kms_key_arn
}

# IAM Role for Lambda
resource "aws_iam_role" "lambda_execution" {
  name               = "${var.project_name}-lambda-role"
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

# IAM Role for AWS Bedrock AgentCore Gateway
resource "aws_iam_role" "gateway_role" {
  name = "${var.project_name}-gateway-role"

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

  tags = var.common_tags
}

# Policy allowing AWS Bedrock AgentCore Gateway to invoke Lambda
resource "aws_iam_role_policy" "gateway_lambda_invoke" {
  name = "${var.project_name}-gateway-lambda-invoke"
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

# AWS Bedrock AgentCore Gateway with JWT authorization
resource "aws_bedrockagentcore_gateway" "main" {
  name            = var.project_name
  protocol_type   = "MCP"
  role_arn        = aws_iam_role.gateway_role.arn
  authorizer_type = "CUSTOM_JWT"

  authorizer_configuration {
    custom_jwt_authorizer {
      discovery_url    = local.entra_discovery_url
      allowed_audience = local.gateway_allowed_audiences
    }
  }

  tags = var.common_tags
}

# AWS Bedrock AgentCore Gateway Target (Lambda)
# Tool schemas loaded directly from programmatically generated tool_schema.json
resource "aws_bedrockagentcore_gateway_target" "lambda" {
  name               = "${var.project_name}-target"
  gateway_identifier = aws_bedrockagentcore_gateway.main.gateway_id
  description        = "Lambda target with MCP tools from tool_schema.json"

  target_configuration {
    mcp {
      lambda {
        lambda_arn = aws_lambda_function.bedrock_agent_gateway.arn

        # Load tool schemas from tool_schema.json using dynamic blocks
        dynamic "tool_schema" {
          for_each = jsondecode(file(local.tool_schema_path))
          content {
            inline_payload {
              name        = tool_schema.value.name
              description = tool_schema.value.description

              input_schema {
                type        = tool_schema.value.inputSchema.type
                description = try(tool_schema.value.inputSchema.description, null)

                # Dynamically create property blocks from inputSchema.properties
                dynamic "property" {
                  for_each = try(tool_schema.value.inputSchema.properties, {})
                  content {
                    name        = property.key
                    type        = property.value.type
                    description = try(property.value.description, null)
                    required    = contains(try(tool_schema.value.inputSchema.required, []), property.key)
                  }
                }
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

# Lambda permission for AWS Bedrock AgentCore Gateway to invoke
resource "aws_lambda_permission" "agentcore_gateway_invoke" {
  statement_id  = "AllowAgentCoreGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.bedrock_agent_gateway.function_name
  principal     = "bedrock.amazonaws.com"
  source_arn    = aws_bedrockagentcore_gateway.main.gateway_arn
}

# Optional: Secrets Manager Access
resource "aws_iam_role_policy" "secrets_manager" {
  count = length(var.secrets_manager_arns) > 0 ? 1 : 0

  name   = "${var.project_name}-secrets-manager"
  role   = aws_iam_role.lambda_execution.id
  policy = data.aws_iam_policy_document.secrets_manager[0].json
}

data "aws_iam_policy_document" "secrets_manager" {
  count = length(var.secrets_manager_arns) > 0 ? 1 : 0

  statement {
    effect = "Allow"
    actions = [
      "secretsmanager:GetSecretValue",
    ]
    resources = var.secrets_manager_arns
  }
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
