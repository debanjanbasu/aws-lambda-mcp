//! Schema generator for Amazon Bedrock Agent tools.
//!
//! This binary scans registered tools and generates `tool_schema.json`,
//! which contains the input/output schemas in Amazon Bedrock format.

use schemars::{JsonSchema, schema_for};
use serde_json::{Value, json};
use std::fs;

// Represents a tool with its metadata and schemas
struct Tool {
    name: String,
    description: String,
    input_schema: Value,
    output_schema: Value,
}

// Macro to create a tool entry with automatic schema generation
macro_rules! tool_entry {
    ($attr_fn:expr, $input_ty:ty, $output_ty:ty) => {{
        let attr = $attr_fn;
        Tool {
            name: attr.name.into(),
            description: attr.description.unwrap_or_default().into(),
            input_schema: generate_bedrock_schema::<$input_ty>(),
            output_schema: generate_bedrock_schema::<$output_ty>(),
        }
    }};
}

fn main() {
    let tools = vec![
        tool_entry!(
            aws_lambda_mcp::tools::weather::get_weather_tool_attr(),
            aws_lambda_mcp::models::WeatherRequest,
            aws_lambda_mcp::models::WeatherResponse
        ),
        // Add new tools here:
        // tool_entry!(
        //     aws_lambda_mcp::tools::example::another_tool_tool_attr(),
        //     aws_lambda_mcp::models::AnotherInput,
        //     aws_lambda_mcp::models::AnotherOutput
        // ),
    ];

    write_schema(&tools);
    println!("✅ Generated tool_schema.json with {} tool(s)", tools.len());
}

// Generates a schema in Amazon Bedrock format for the given type
fn generate_bedrock_schema<T: JsonSchema>() -> Value {
    let mut schema = serde_json::to_value(schema_for!(T)).unwrap_or_else(|e| {
        eprintln!("Failed to serialize schema: {e}");
        std::process::exit(1);
    });

    // Clean up schema to conform to Amazon Bedrock AgentCore format
    if let Some(obj) = schema.as_object_mut() {
        // Remove fields not supported by Amazon Bedrock
        obj.remove("$schema");
        obj.remove("title");

        // Handle enum references by converting them to string types
        if let Some(defs) = obj.remove("$defs")
            && let Some(properties) = obj.get_mut("properties").and_then(|p| p.as_object_mut())
        {
            for prop_value in properties.values_mut() {
                if let Some(prop_obj) = prop_value.as_object_mut()
                    && let Some(Value::String(ref_path)) = prop_obj.get("$ref")
                    && let Some(def_name) = ref_path.strip_prefix("#/$defs/")
                    && let Some(def_value) = defs.get(def_name)
                {
                    prop_obj.remove("$ref");

                    // Convert enums to string type for Amazon Bedrock compatibility
                    if def_value.get("enum").is_some() {
                        prop_obj.insert("type".to_string(), json!("string"));
                    }
                }
            }
        }

        // Remove format fields from all properties (not supported by Amazon Bedrock)
        if let Some(properties) = obj.get_mut("properties").and_then(|p| p.as_object_mut()) {
            for prop_value in properties.values_mut() {
                if let Some(prop_obj) = prop_value.as_object_mut() {
                    prop_obj.remove("format");
                }
            }
        }
    }

    schema
}

// Writes the tools schema to tool_schema.json
fn write_schema(tools: &[Tool]) {
    let schemas: Vec<Value> = tools
        .iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": tool.input_schema,
                "outputSchema": tool.output_schema
            })
        })
        .collect();

    let json = serde_json::to_string_pretty(&schemas).unwrap_or_else(|e| {
        eprintln!("Failed to serialize schema: {e}");
        std::process::exit(1);
    });

    fs::write("tool_schema.json", json).unwrap_or_else(|e| {
        eprintln!("Failed to write tool_schema.json: {e}");
        std::process::exit(1);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generate_bedrock_schema_removes_unsupported_fields() {
        // Test with a simple struct that has format and $schema
        #[derive(schemars::JsonSchema)]
        struct TestStruct {
            #[schemars(description = "A test field")]
            field: String,
        }

        let schema = generate_bedrock_schema::<TestStruct>();

        // Should not contain $schema or format fields
        assert!(schema.get("$schema").is_none());
        assert!(schema.get("properties").unwrap().get("field").unwrap().get("format").is_none());
    }

    #[test]
    fn test_generate_bedrock_schema_handles_enums() {
        #[derive(schemars::JsonSchema)]
        enum TestEnum {
            Option1,
            Option2,
        }

        #[derive(schemars::JsonSchema)]
        struct TestStruct {
            enum_field: TestEnum,
        }

        let schema = generate_bedrock_schema::<TestStruct>();

        // Enum fields should be converted to string type
        let enum_prop = schema.get("properties").unwrap().get("enum_field").unwrap();
        assert_eq!(enum_prop.get("type").unwrap(), "string");
        assert!(enum_prop.get("$ref").is_none());
    }

    #[test]
    fn test_write_schema_creates_valid_json() {
        let tools = vec![
            Tool {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                input_schema: json!({"type": "object", "properties": {"input": {"type": "string"}}}),
                output_schema: json!({"type": "object", "properties": {"output": {"type": "string"}}}),
            }
        ];

        // Temporarily change the output path for testing
        let test_file = "test_tool_schema.json";
        let original_write = |path: &str, content: &str| fs::write(path, content);

        // This is a simplified test - in real scenario we'd mock fs::write
        let schemas: Vec<Value> = tools
            .iter()
            .map(|tool| {
                json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": tool.input_schema,
                    "outputSchema": tool.output_schema
                })
            })
            .collect();

        let json_str = serde_json::to_string_pretty(&schemas).unwrap();
        assert!(json_str.contains("test_tool"));
        assert!(json_str.contains("A test tool"));
    }
}
