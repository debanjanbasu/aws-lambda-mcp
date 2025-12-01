# Lambda Functions, Log Groups, and Dead Letter Queue
# Infrastructure for serverless compute resources

# SQS Dead Letter Queue for Lambda
resource "aws_sqs_queue" "lambda_dlq" {
  name                       = "${local.project_name_with_suffix}-dlq"
  message_retention_seconds  = 259200  # 3 days (259200 seconds)
  visibility_timeout_seconds = 30

  # Enable server-side encryption using AWS managed KMS key for SQS (free tier)
  kms_master_key_id = "alias/aws/sqs"

  tags = var.common_tags
}

# Lambda Function
resource "aws_lambda_function" "bedrock_agentcore_gateway_main_lambda" {
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