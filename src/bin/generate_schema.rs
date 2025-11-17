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
    let mut schema = serde_json::to_value(schema_for!(T)).expect("Failed to serialize schema");

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

    let json = serde_json::to_string_pretty(&schemas).expect("Failed to serialize schema");

    fs::write("tool_schema.json", json).expect("Failed to write tool_schema.json");
}
