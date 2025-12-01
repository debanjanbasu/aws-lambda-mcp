# Amazon Bedrock AgentCore Gateway Infrastructure
# Main entry point - data sources used across other files
#
# File structure:
# - main.tf      : Data sources (this file)
# - lambda.tf    : Lambda functions, log groups, DLQ
# - iam.tf       : IAM roles and policies
# - gateway.tf   : Bedrock AgentCore Gateway and target configuration
# - entra_oauth.tf : Entra ID OAuth application
# - locals.tf    : Local values and computed expressions
# - variables.tf : Input variables
# - outputs.tf   : Output values
# - providers.tf : Provider configurations
# - backend.tf   : Terraform state backend

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
