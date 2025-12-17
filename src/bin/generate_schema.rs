//! Schema generator for Amazon Bedrock Agent tools.
//!
//! This binary scans registered tools and generates `tool_schema.json`,
//! which contains the input/output schemas in Amazon Bedrock format.

use aws_lambda_mcp::models::personalized::{
    PersonalizedGreetingRequest, PersonalizedGreetingResponse,
};
use aws_lambda_mcp::models::weather::{WeatherRequest, WeatherResponse};
use schemars::{JsonSchema, schema_for};
use serde_json::{Value, json, to_string_pretty, to_value};
use std::fs::write;
use std::process::exit;

// Represents a tool with its metadata and schemas
struct Tool {
    name: String,
    description: String,
    input_schema: Value,
    output_schema: Value,
}

fn main() {
    let tools = vec![
        Tool {
            name: "get_weather".to_string(),
            description: "Fetches weather data from the Open-Meteo API.".to_string(),
            input_schema: generate_bedrock_schema::<WeatherRequest>(),
            output_schema: generate_bedrock_schema::<WeatherResponse>(),
        },
        Tool {
            name: "get_personalized_greeting".to_string(),
            description: "Generates a personalized greeting for a user.".to_string(),
            input_schema: generate_bedrock_schema::<PersonalizedGreetingRequest>(),
            output_schema: generate_bedrock_schema::<PersonalizedGreetingResponse>(),
        },
    ];

    write_schema(&tools);
    println!("✅ Generated tool_schema.json with {} tool(s)", tools.len());
}

// Generates a schema in Amazon Bedrock format for the given type
fn generate_bedrock_schema<T: JsonSchema>() -> Value {
    let mut schema = to_value(schema_for!(T)).unwrap_or_else(|e| {
        eprintln!("Failed to serialize schema: {e}");
        exit(1);
    });

    // Clean up schema to conform to Amazon Bedrock AgentCore format
    if let Some(obj) = schema.as_object_mut() {
        // Remove fields not supported by Amazon Bedrock
        obj.remove("$schema");
        obj.remove("title");

        if let Some(defs) = obj.remove("$defs")
            && let Some(properties) = obj.get_mut("properties").and_then(|p| p.as_object_mut())
        {
            for (_prop_name, prop_value) in properties.iter_mut() {
                if let Some(prop_obj) = prop_value.as_object_mut()
                    && let Some(Value::String(ref_path)) = prop_obj.get("$ref")
                    && let Some(def_name) = ref_path.strip_prefix("#/$defs/")
                    && let Some(def_value) = defs.get(def_name)
                {
                    // Inline the definition instead of keeping the reference
                    if let Some(def_obj) = def_value.as_object() {
                        prop_obj.clear();
                        prop_obj.extend(def_obj.clone());
                    }

                    // Convert enums to string type for Amazon Bedrock compatibility
                    if def_value.get("enum").is_some() {
                        prop_obj.insert("type".to_string(), json!("string"));
                    }
                }
            }
        }

        // Remove format fields and convert union types to primary type
        if let Some(properties) = obj.get_mut("properties").and_then(|p| p.as_object_mut()) {
            // Remove fields that are injected by the interceptor
            properties.remove("user_id");
            properties.remove("user_name");

            for prop_value in properties.values_mut() {
                if let Some(prop_obj) = prop_value.as_object_mut() {
                    prop_obj.remove("format");

                    // Convert union types like ["string", "null"] to just "string"
                    if let Some(type_value) = prop_obj.get("type")
                        && let Some(type_array) = type_value.as_array()
                        && type_array.len() == 2
                        && type_array.contains(&json!("null"))
                    {
                        for t in type_array {
                            if t != &json!("null") {
                                prop_obj.insert("type".to_string(), t.clone());
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Remove injected fields from required fields since they're provided by interceptor
        if let Some(required) = obj.get_mut("required").and_then(|r| r.as_array_mut()) {
            required.retain(|item| item != "user_id" && item != "user_name");
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

    let json = to_string_pretty(&schemas).unwrap_or_else(|e| {
        eprintln!("Failed to serialize schema: {e}");
        exit(1);
    });

    write("tool_schema.json", json).unwrap_or_else(|e| {
        eprintln!("Failed to write tool_schema.json: {e}");
        exit(1);
    });
}
