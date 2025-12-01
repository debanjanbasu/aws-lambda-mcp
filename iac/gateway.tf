# Amazon Bedrock AgentCore Gateway
# MCP server with JWT authorization and semantic tool search

# Amazon Bedrock AgentCore Gateway with JWT authorization
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
        lambda_arn = aws_lambda_function.bedrock_agentcore_gateway_main_lambda.arn

        # Load tool schemas from tool_schema.json using dynamic blocks
        tool_schema {
          dynamic "inline_payload" {
            for_each = jsondecode(file(local.tool_schema_path))
            content {
              name        = inline_payload.value.name
              description = inline_payload.value.description

              input_schema {
                type        = inline_payload.value.inputSchema.type
                description = try(inline_payload.value.inputSchema.description, null)

                # Dynamically create property blocks from inputSchema.properties
                dynamic "property" {
                  for_each = try(inline_payload.value.inputSchema.properties, {})
                  content {
                    name        = property.key
                    type        = property.value.type
                    description = try(property.value.description, null)
                    required    = contains(try(inline_payload.value.inputSchema.required, []), property.key)
                  }
                }
              }

              output_schema {
                type        = inline_payload.value.outputSchema.type
                description = try(inline_payload.value.outputSchema.description, null)

                # Dynamically create property blocks from outputSchema.properties
                dynamic "property" {
                  for_each = try(inline_payload.value.outputSchema.properties, {})
                  content {
                    name        = property.key
                    type        = property.value.type
                    description = try(property.value.description, null)
                    required    = contains(try(inline_payload.value.outputSchema.required, []), property.key)
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

# Lambda permission for Amazon Bedrock AgentCore Gateway to invoke main Lambda
resource "aws_lambda_permission" "agentcore_gateway_invoke" {
  statement_id  = "AllowAgentCoreGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.bedrock_agentcore_gateway_main_lambda.function_name
  principal     = "bedrock.amazonaws.com"
  source_arn    = aws_bedrockagentcore_gateway.main.gateway_arn
}

# Lambda permission for Amazon Bedrock AgentCore Gateway to invoke interceptor Lambda
resource "aws_lambda_permission" "agentcore_gateway_interceptor_invoke" {
  statement_id  = "AllowAgentCoreGatewayInterceptorInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.gateway_interceptor.function_name
  principal     = "bedrock-agentcore.amazonaws.com"
  source_arn    = aws_bedrockagentcore_gateway.main.gateway_arn
}

# Additional permission for interceptor with bedrock.amazonaws.com principal
resource "aws_lambda_permission" "agentcore_gateway_interceptor_invoke_bedrock" {
  statement_id  = "AllowBedrockGatewayInterceptorInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.gateway_interceptor.function_name
  principal     = "bedrock.amazonaws.com"
  source_arn    = aws_bedrockagentcore_gateway.main.gateway_arn
}

# Configure interceptor on gateway via AWS CLI
# The Terraform AWS provider doesn't yet support interceptor_configurations,
# so we use a null_resource with local-exec to configure it via AWS CLI.
resource "null_resource" "configure_gateway_interceptor" {
  # Re-run when gateway or interceptor Lambda changes
  triggers = {
    gateway_id      = aws_bedrockagentcore_gateway.main.gateway_id
    interceptor_arn = aws_lambda_function.gateway_interceptor.arn
  }

  provisioner "local-exec" {
    command = <<-EOT
      echo "Configuring interceptor on gateway..."

      # Get current gateway configuration
      GATEWAY_CONFIG=$(aws bedrock-agentcore-control get-gateway \
        --gateway-identifier ${aws_bedrockagentcore_gateway.main.gateway_id} \
        --output json)

      # Extract required fields
      ROLE_ARN=$(echo "$GATEWAY_CONFIG" | jq -r '.roleArn')
      PROTOCOL_TYPE=$(echo "$GATEWAY_CONFIG" | jq -r '.protocolType')
      AUTHORIZER_TYPE=$(echo "$GATEWAY_CONFIG" | jq -r '.authorizerType')
      PROTOCOL_CONFIG=$(echo "$GATEWAY_CONFIG" | jq -c '.protocolConfiguration')
      AUTHORIZER_CONFIG=$(echo "$GATEWAY_CONFIG" | jq -c '.authorizerConfiguration')
      EXCEPTION_LEVEL=$(echo "$GATEWAY_CONFIG" | jq -r '.exceptionLevel // empty')

      # Build the update command
      UPDATE_CMD="aws bedrock-agentcore-control update-gateway \
        --gateway-identifier ${aws_bedrockagentcore_gateway.main.gateway_id} \
        --name ${aws_bedrockagentcore_gateway.main.name} \
        --role-arn $ROLE_ARN \
        --protocol-type $PROTOCOL_TYPE \
        --authorizer-type $AUTHORIZER_TYPE \
        --protocol-configuration '$PROTOCOL_CONFIG' \
        --authorizer-configuration '$AUTHORIZER_CONFIG' \
        --interceptor-configurations '[{\"interceptor\":{\"lambda\":{\"arn\":\"${aws_lambda_function.gateway_interceptor.arn}\"}},\"interceptionPoints\":[\"REQUEST\"],\"inputConfiguration\":{\"passRequestHeaders\":true}}]'"

      # Add exception level if set
      if [ -n "$EXCEPTION_LEVEL" ]; then
        UPDATE_CMD="$UPDATE_CMD --exception-level $EXCEPTION_LEVEL"
      fi

      # Execute the update
      eval $UPDATE_CMD

      # Wait for gateway to be ready
      echo "Waiting for gateway to be ready..."
      for i in {1..30}; do
        STATUS=$(aws bedrock-agentcore-control get-gateway \
          --gateway-identifier ${aws_bedrockagentcore_gateway.main.gateway_id} \
          --query 'status' --output text)
        if [ "$STATUS" = "READY" ]; then
          echo "Gateway is ready with interceptor configured"
          exit 0
        fi
        sleep 2
      done
      echo "Warning: Gateway did not reach READY state within timeout"
    EOT
  }

  depends_on = [
    aws_bedrockagentcore_gateway.main,
    aws_bedrockagentcore_gateway_target.lambda,
    aws_lambda_function.gateway_interceptor,
    aws_lambda_permission.agentcore_gateway_invoke,
    aws_lambda_permission.agentcore_gateway_interceptor_invoke,
    aws_lambda_permission.agentcore_gateway_interceptor_invoke_bedrock
  ]
}