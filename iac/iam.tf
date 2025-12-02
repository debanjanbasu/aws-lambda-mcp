# IAM Roles and Policies
# Identity and access management for Lambda and Gateway services
#
# Security considerations:
# - Assume role policies are kept simple to ensure compatibility with AWS services
# - Resource-based policies are scoped to specific ARNs where possible
# - No wildcard (*) resources except where required by AWS managed policies

# -----------------------------------------------------------------------------
# Assume Role Policies
# -----------------------------------------------------------------------------

# Assume role policy for Lambda functions
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

# Assume role policy for Bedrock AgentCore Gateway
# Note: Bedrock services may not pass aws:SourceAccount/aws:SourceRegion conditions
# so we keep the trust policy simple to ensure the gateway can assume the role
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

# -----------------------------------------------------------------------------
# Main Lambda Role
# -----------------------------------------------------------------------------

resource "aws_iam_role" "lambda_execution" {
  name               = "${local.project_name_with_suffix}-lambda-role"
  assume_role_policy = data.aws_iam_policy_document.lambda_assume_role.json
  tags               = var.common_tags
}

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

# -----------------------------------------------------------------------------
# Bedrock Gateway Role
# -----------------------------------------------------------------------------

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
