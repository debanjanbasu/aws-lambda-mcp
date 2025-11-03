# Bedrock Agent Gateway Infrastructure
# Main infrastructure resources for AWS Lambda and Bedrock integration

# Create zip file from Lambda binary
data "archive_file" "lambda_zip" {
  type        = "zip"
  source_file = var.lambda_binary_path
  output_path = "${path.module}/.terraform/lambda-${filemd5(var.lambda_binary_path)}.zip"
}

# Lambda Function
resource "aws_lambda_function" "bedrock_agent_gateway" {
  function_name = var.lambda_function_name
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
  name              = "/aws/lambda/${var.lambda_function_name}"
  retention_in_days = var.log_retention_days

  # KMS encryption disabled to reduce costs (free tier uses SSE)
  # kms_key_id = var.cloudwatch_kms_key_arn
}

# IAM Role for Lambda
resource "aws_iam_role" "lambda_execution" {
  name               = "${var.lambda_function_name}-execution-role"
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

# Bedrock Agent Gateway
resource "aws_bedrockagentcore_gateway" "main" {
  gateway_name = var.gateway_name
  description  = "Bedrock Agent Gateway for tool invocations with Entra ID OAuth"

  # OAuth authorization using Entra ID discovery URL
  authorizer_configuration {
    openid_connect {
      issuer_url = "https://login.microsoftonline.com/${data.azuread_client_config.current.tenant_id}/v2.0"
      audience   = azuread_application.bedrock_gateway.client_id
    }
  }

  tags = var.common_tags
}

# Bedrock Agent Gateway Target (Lambda)
resource "aws_bedrockagentcore_gateway_target" "lambda" {
  gateway_identifier = aws_bedrockagentcore_gateway.main.gateway_id

  target {
    lambda_target {
      lambda_arn = aws_lambda_function.bedrock_agent_gateway.arn
    }
  }

  # Tool schema from tool_schema.json
  tool_schema {
    tools = jsondecode(file(var.tool_schema_path))
  }
}

# Lambda permission for Bedrock Agent Gateway to invoke
resource "aws_lambda_permission" "bedrock_gateway" {
  statement_id  = "AllowBedrockAgentGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.bedrock_agent_gateway.function_name
  principal     = "bedrock.amazonaws.com"
  source_arn    = aws_bedrockagentcore_gateway.main.gateway_arn
}

# Optional: Secrets Manager Access
resource "aws_iam_role_policy" "secrets_manager" {
  count = length(var.secrets_manager_arns) > 0 ? 1 : 0

  name   = "${var.lambda_function_name}-secrets-manager"
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
