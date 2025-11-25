# Amazon Bedrock AgentCore Gateway Infrastructure
# Main infrastructure resources for AWS Lambda and Amazon Bedrock AgentCore integration

# Create zip file from Lambda binary
# Use a conditional expression to avoid evaluating filemd5 when the file doesn't exist
data "archive_file" "lambda_zip" {
  type        = "zip"
  source_file = local.lambda_binary_path
  output_path = "${path.module}/.terraform/lambda.zip"
}

# Create zip file from interceptor Lambda binary
data "archive_file" "interceptor_lambda_zip" {
  type        = "zip"
  source_file = local.interceptor_lambda_binary_path
  output_path = "${path.module}/.terraform/interceptor-lambda.zip"
}

# Get current AWS account information
data "aws_caller_identity" "current" {}

# SNS Topic for CloudFormation stack notifications
resource "aws_sns_topic" "cloudformation_notifications" {
  name = "${local.project_name_with_suffix}-cfn-notifications"

  # Enable server-side encryption using AWS managed SNS key (free)
  kms_master_key_id = "alias/aws/sns"

  tags = var.common_tags
}

# SNS Topic Policy for CloudFormation notifications
resource "aws_sns_topic_policy" "cloudformation_notifications" {
  arn = aws_sns_topic.cloudformation_notifications.arn

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Principal = {
          Service = "cloudformation.amazonaws.com"
        }
        Action   = "SNS:Publish"
        Resource = aws_sns_topic.cloudformation_notifications.arn
        Condition = {
          StringEquals = {
            "AWS:SourceAccount" = data.aws_caller_identity.current.account_id
          }
        }
      }
    ]
  })
}

# SQS Dead Letter Queue for Lambda
resource "aws_sqs_queue" "lambda_dlq" {
  name                       = "${local.project_name_with_suffix}-dlq"
  message_retention_seconds  = 1209600  # 14 days
  visibility_timeout_seconds = 30

  # Enable server-side encryption using AWS managed KMS key for SQS (free tier)
  kms_master_key_id = "alias/aws/sqs"

  tags = var.common_tags
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

  memory_size                    = var.lambda_memory_size
  timeout                        = var.lambda_timeout
  reserved_concurrent_executions = var.lambda_concurrent_executions

  # Dead Letter Queue configuration
  dead_letter_config {
    target_arn = aws_sqs_queue.lambda_dlq.arn
  }

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

# Interceptor Lambda Function
resource "aws_lambda_function" "gateway_interceptor" {
  function_name = "${local.project_name_with_suffix}-interceptor"
  role          = aws_iam_role.lambda_execution.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["arm64"]

  filename         = data.archive_file.interceptor_lambda_zip.output_path
  source_code_hash = data.archive_file.interceptor_lambda_zip.output_base64sha256

  memory_size                    = 128  # Minimum memory for cost optimization
  timeout                        = 30   # Standard timeout for consistency
  reserved_concurrent_executions = var.lambda_concurrent_executions * 2

  # Dead Letter Queue configuration
  dead_letter_config {
    target_arn = aws_sqs_queue.lambda_dlq.arn
  }

  environment {
    variables = {
      RUST_LOG = var.rust_log_level
    }
  }

  depends_on = [
    aws_iam_role_policy_attachment.lambda_basic,
    aws_cloudwatch_log_group.interceptor_lambda_logs,
  ]
}

# CloudWatch Log Group
resource "aws_cloudwatch_log_group" "lambda_logs" {
  name              = "/aws/lambda/${local.project_name_with_suffix}"
  retention_in_days = var.log_retention_days

  # KMS encryption disabled to reduce costs (free tier uses SSE)
  # kms_key_id = var.cloudwatch_kms_key_arn
}

# CloudWatch Log Group for Interceptor Lambda
resource "aws_cloudwatch_log_group" "interceptor_lambda_logs" {
  name              = "/aws/lambda/${local.project_name_with_suffix}-interceptor"
  retention_in_days = var.log_retention_days
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

# SQS DLQ Policy for Lambda
resource "aws_iam_role_policy" "lambda_sqs_dlq" {
  name = "${local.project_name_with_suffix}-lambda-sqs-dlq"
  role = aws_iam_role.lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = [
        "sqs:SendMessage"
      ]
      Resource = aws_sqs_queue.lambda_dlq.arn
    }]
  })
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
# Tool schemas loaded directly from programmatically generated tool_schema.json
resource "aws_bedrockagentcore_gateway_target" "lambda" {
  name               = "${local.project_name_with_suffix}-target"
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

              output_schema {
                type        = tool_schema.value.outputSchema.type
                description = try(tool_schema.value.outputSchema.description, null)

                # Dynamically create property blocks from outputSchema.properties
                dynamic "property" {
                  for_each = try(tool_schema.value.outputSchema.properties, {})
                  content {
                    name        = property.key
                    type        = property.value.type
                    description = try(property.value.description, null)
                    required    = contains(try(tool_schema.value.outputSchema.required, []), property.key)
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

# Lambda permission for Amazon Bedrock AgentCore Gateway to invoke
resource "aws_lambda_permission" "agentcore_gateway_invoke" {
  statement_id  = "AllowAgentCoreGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.bedrock_agent_gateway.function_name
  principal     = "bedrock.amazonaws.com"
  source_arn    = aws_bedrockagentcore_gateway.main.gateway_arn
}

# CloudFormation stack to add interceptor to the gateway
resource "aws_cloudformation_stack" "gateway_interceptor" {
  name         = "${local.project_name_with_suffix}-interceptor"
  template_body = file("${path.module}/gateway-with-interceptor.yaml")

  parameters = {
    GatewayId           = aws_bedrockagentcore_gateway.main.gateway_id
    InterceptorLambdaArn = aws_lambda_function.gateway_interceptor.arn
  }

  # Send CloudFormation events to SNS topic
  notification_arns = [aws_sns_topic.cloudformation_notifications.arn]

  depends_on = [
    aws_bedrockagentcore_gateway.main,
    aws_lambda_function.gateway_interceptor,
    aws_sns_topic_policy.cloudformation_notifications,
  ]
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
