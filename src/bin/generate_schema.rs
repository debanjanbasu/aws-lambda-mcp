//! Schema generator for AWS Bedrock Agent tools.
//!
//! This binary scans registered tools and generates `tool_schema.json`,
//! which contains the input/output schemas in AWS Bedrock format.
//!
//! # Adding New Tools
//!
//! When you create a new tool in `src/tools/`:
//! 1. Annotate it with `#[tool(...)]` from rmcp
//! 2. Add an entry to the `tools` vec in `main()`
//! 3. Run `make schema` to regenerate `tool_schema.json`
//!
//! The `tool_entry!` macro automatically extracts tool metadata and schemas,
//! so adding a tool is just one line.

use schemars::{JsonSchema, schema_for};
use serde_json::{Value, json};
use std::fs;

struct Tool {
    name: String,
    description: String,
    input_schema: Value,
    output_schema: Value,
}

/// Helper macro to extract tool metadata and generate schemas.
/// Reduces boilerplate when adding new tools to the registry.
macro_rules! tool_entry {
    ($attr_fn:expr, $input_ty:ty, $output_ty:ty) => {
        {
            let attr = $attr_fn;
            Tool {
                name: attr.name.to_string(),
                description: attr.description.map_or_else(String::new, String::from),
                input_schema: generate_bedrock_schema::<$input_ty>(),
                output_schema: generate_bedrock_schema::<$output_ty>(),
            }
        }
    };
}

fn main() {
    // Tool registry: Add new tools here as they are created in src/tools/
    // Format: tool_entry!(module::function_tool_attr(), InputType, OutputType)
    let tools = vec![
        tool_entry!(
            aws_lambda_mcp::tools::weather::get_weather_tool_attr(),
            aws_lambda_mcp::models::WeatherRequest,
            aws_lambda_mcp::models::WeatherResponse
        ),
        // Add new tools below (one line per tool):
        // tool_entry!(
        //     aws_lambda_mcp::tools::example::another_tool_tool_attr(),
        //     aws_lambda_mcp::models::AnotherInput,
        //     aws_lambda_mcp::models::AnotherOutput
        // ),
    ];

    write_schema(&tools);
    println!("✅ Generated tool_schema.json with {} tool(s)", tools.len());
}

fn generate_bedrock_schema<T: JsonSchema>() -> Value {
    let mut schema = serde_json::to_value(schema_for!(T)).unwrap_or_else(|e| {
        eprintln!("Failed to serialize schema: {e}");
        std::process::exit(1);
    });
    cleanup_bedrock_schema(&mut schema);
    schema
}

fn cleanup_bedrock_schema(schema: &mut Value) {
    let Some(obj) = schema.as_object_mut() else {
        return;
    };

    obj.remove("$schema");
    obj.remove("title");

    if let Some(defs) = obj.remove("$defs") {
        inline_enum_refs(obj, &defs);
    }
}

fn inline_enum_refs(schema: &mut serde_json::Map<String, Value>, defs: &Value) {
    let Some(properties) = schema.get_mut("properties").and_then(|p| p.as_object_mut()) else {
        return;
    };

    for prop_value in properties.values_mut() {
        let Some(prop_obj) = prop_value.as_object_mut() else {
            continue;
        };

        if let Some(Value::String(ref_path)) = prop_obj.get("$ref")
            && let Some(def_name) = ref_path.strip_prefix("#/$defs/")
            && let Some(def_value) = defs.get(def_name)
        {
            prop_obj.remove("$ref");

            // AWS Bedrock doesn't support enums, convert to string
            if def_value.get("enum").is_some() {
                prop_obj.insert("type".to_string(), json!("string"));
            }
        }
    }
}

fn write_schema(tools: &[Tool]) {
    let schemas: Vec<_> = tools
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
