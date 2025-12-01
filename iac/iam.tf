# IAM Roles and Policies
# Identity and access management for Lambda and Gateway services

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
  name               = "${local.project_name_with_suffix}-gateway-role"
  assume_role_policy = data.aws_iam_policy_document.gateway_assume_role.json
  tags               = var.common_tags
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
      Resource = aws_lambda_function.bedrock_agentcore_gateway_main_lambda.arn
    }]
  })
}