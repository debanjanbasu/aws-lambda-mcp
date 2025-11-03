# Tool Schema Integration

The Bedrock Gateway Target automatically loads tool schemas from `tool_schema.json`.

## How It Works

### 1. Schema Generation (Rust)

```bash
make schema  # Generates tool_schema.json from Rust code
```

Creates:
```json
[
  {
    "name": "get_weather",
    "description": "Get current weather...",
    "inputSchema": {
      "type": "object",
      "properties": {
        "location": {
          "type": "string",
          "description": "Location name..."
        }
      },
      "required": ["location"]
    }
  }
]
```

### 2. Terraform Reads Schema

```hcl
dynamic "tool_schema" {
  for_each = jsondecode(file(var.tool_schema_path))
  content {
    inline_payload {
      name        = tool_schema.value.name
      description = tool_schema.value.description

      input_schema {
        type            = tool_schema.value.inputSchema.type
        description     = try(tool_schema.value.inputSchema.description, null)
        properties_json = jsonencode(tool_schema.value.inputSchema.properties)
      }
    }
  }
}
```

### 3. Gateway Uses Schema

- Bedrock validates requests against the schema
- Only valid tool calls reach Lambda
- Full type checking before invocation

## Workflow

```bash
# 1. Add new tool in Rust
vim src/tools/my_tool.rs

# 2. Register tool
vim src/bin/generate_schema.rs

# 3. Regenerate schema
make schema

# 4. Deploy updated gateway
cd iac
terraform apply
```

## Schema Mapping

| tool_schema.json | Terraform HCL | Gateway |
|------------------|---------------|---------|
| `name` | `name` | Tool name |
| `description` | `description` | Tool description |
| `inputSchema.type` | `input_schema.type` | Schema type |
| `inputSchema.properties` | `properties_json` | JSON properties |
| `inputSchema.required` | (in properties_json) | Required fields |

## Benefits

✅ **Single source of truth**: Schema defined once in Rust  
✅ **Type safety**: Rust types → JSON Schema → Gateway validation  
✅ **Automatic updates**: `make schema` → `terraform apply`  
✅ **No manual sync**: Terraform reads JSON directly  
✅ **Validation at gateway**: Invalid requests blocked before Lambda  

## Properties JSON

The `properties_json` attribute accepts the full JSON Schema properties object:

```json
{
  "location": {
    "type": "string",
    "description": "City name"
  },
  "units": {
    "type": "string",
    "enum": ["celsius", "fahrenheit"]
  }
}
```

This is automatically extracted from `tool_schema.json` and passed to the gateway.

## Adding New Tools

1. **Create Rust tool**:
   ```rust
   #[tool(description = "My new tool")]
   pub async fn my_tool(request: MyRequest) -> Result<MyResponse> { ... }
   ```

2. **Register in schema generator**:
   ```rust
   tool_entry!(
       aws_lambda_mcp::tools::my_tool::my_tool_tool_attr(),
       aws_lambda_mcp::models::MyRequest,
       aws_lambda_mcp::models::MyResponse
   )
   ```

3. **Regenerate and deploy**:
   ```bash
   make schema
   cd iac && terraform apply
   ```

Done! The new tool is available in the gateway.

## Testing Schema

```bash
# Validate JSON syntax
jq . tool_schema.json

# Check Terraform reads it correctly
cd iac
terraform console
> jsondecode(file("../tool_schema.json"))

# Plan to see what changes
terraform plan
```

## Troubleshooting

### Error: "Invalid JSON"
```bash
# Fix JSON syntax
jq . tool_schema.json
make schema  # Regenerate
```

### Error: "properties_json must be valid JSON"
- Ensure `inputSchema.properties` exists in tool_schema.json
- Check for proper JSON encoding

### Tool not appearing in gateway
```bash
# Verify schema includes the tool
jq '.[].name' tool_schema.json

# Redeploy
terraform apply
```

---

**Last Updated**: 2025-11-03  
**Related**: See `src/bin/generate_schema.rs` for schema generation logic
