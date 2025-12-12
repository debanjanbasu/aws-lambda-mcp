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

fn main() {
    let tools = vec![
        Tool {
            name: "get_weather".to_string(),
            description: "Fetches weather data from the Open-Meteo API.".to_string(),
            input_schema: generate_bedrock_schema::<aws_lambda_mcp::models::weather::WeatherRequest>(
            ),
            output_schema: generate_bedrock_schema::<
                aws_lambda_mcp::models::weather::WeatherResponse,
            >(),
        },
        Tool {
            name: "get_personalized_greeting".to_string(),
            description: "Generates a personalized greeting for a user.".to_string(),
            input_schema: generate_bedrock_schema::<
                aws_lambda_mcp::models::personalized::PersonalizedGreetingRequest,
            >(),
            output_schema: generate_bedrock_schema::<
                aws_lambda_mcp::models::personalized::PersonalizedGreetingResponse,
            >(),
        },
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

        // Flatten nested object properties to string type for Terraform compatibility.
        // The Terraform aws_bedrockagentcore_gateway_target resource's dynamic property blocks
        // don't support defining nested properties for object types. By converting object
        // properties to strings, we ensure the schema is compatible with the infrastructure
        // configuration while maintaining functional correctness in the Lambda handler.
        if let Some(properties) = obj.get_mut("properties").and_then(|p| p.as_object_mut()) {
            for prop_value in properties.values_mut() {
                if let Some(prop_obj) = prop_value.as_object_mut() {
                    if prop_obj.get("type") == Some(&json!("object")) {
                        prop_obj.insert("type".to_string(), json!("string"));
                        prop_obj.remove("properties");
                        prop_obj.remove("required");
                    }
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
