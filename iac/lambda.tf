# Lambda Functions and Supporting Resources
# Main MCP handler and Gateway interceptor Lambda functions

# SQS Dead Letter Queue for Lambda
resource "aws_sqs_queue" "lambda_dlq" {
  name                       = "${local.project_name_with_suffix}-dlq"
  message_retention_seconds  = 1209600 # 14 days
  visibility_timeout_seconds = 30

  # Enable server-side encryption using AWS managed KMS key for SQS (free tier)
  # KMS encryption disabled to reduce costs (free tier uses SSE)
  # kms_master_key_id = "alias/aws/sqs"

  tags = var.common_tags
}

# CloudWatch Log Group for main Lambda
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

# Main Lambda Function - MCP Handler
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

  # Advanced Logging Controls - JSON format for structured logs
  logging_config {
    log_format = "JSON"
    log_group  = aws_cloudwatch_log_group.lambda_logs.name
  }

  environment {
    variables = merge(local.common_lambda_env_vars, var.additional_env_vars)
  }

  depends_on = [
    aws_cloudwatch_log_group.lambda_logs,
  ]
}

# Interceptor Lambda Function - JWT processing and HCM person ID resolution
resource "aws_lambda_function" "gateway_interceptor" {
  function_name = "${local.project_name_with_suffix}-interceptor"
  role          = aws_iam_role.interceptor_lambda_execution.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["arm64"]

  filename         = data.archive_file.interceptor_lambda_zip.output_path
  source_code_hash = data.archive_file.interceptor_lambda_zip.output_base64sha256

  memory_size                    = 128 # Minimum memory for cost optimization
  timeout                        = 30  # Standard timeout for consistency
  reserved_concurrent_executions = var.lambda_concurrent_executions * 2

  # Dead Letter Queue configuration
  dead_letter_config {
    target_arn = aws_sqs_queue.lambda_dlq.arn
  }

  # Advanced Logging Controls - JSON format for structured logs
  logging_config {
    log_format = "JSON"
    log_group  = aws_cloudwatch_log_group.interceptor_lambda_logs.name
  }

  environment {
    variables = merge(local.common_lambda_env_vars, var.additional_env_vars)
  }

  depends_on = [
    aws_cloudwatch_log_group.interceptor_lambda_logs,
  ]
}
