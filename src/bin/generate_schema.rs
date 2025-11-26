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
            description: "Get current weather information for a specified location. Returns temperature (automatically converted to Celsius or Fahrenheit based on the country), WMO weather code, and wind speed in km/h. Supports city names, addresses, or place names worldwide.".to_string(),
            input_schema: generate_bedrock_schema::<aws_lambda_mcp::models::WeatherRequest>(),
            output_schema: generate_bedrock_schema::<aws_lambda_mcp::models::WeatherResponse>(),
        },
        // Add new tools here:
        // Tool {
        //     name: "another_tool".to_string(),
        //     description: "Description of another tool".to_string(),
        //     input_schema: generate_bedrock_schema::<aws_lambda_mcp::models::AnotherInput>(),
        //     output_schema: generate_bedrock_schema::<aws_lambda_mcp::models::AnotherOutput>(),
        // },
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
    use serde_json::Value;

    #[test]
    fn test_generate_bedrock_schema_weather_request() {
        let schema = generate_bedrock_schema::<aws_lambda_mcp::models::WeatherRequest>();
        let schema_value: Value = serde_json::from_value(schema).unwrap();

        // Check that required fields are present
        assert!(schema_value.get("type").is_some());
        assert!(schema_value.get("properties").is_some());

        // Check that location field exists
        let properties = schema_value.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("location"));

        // Check that format field is removed (Bedrock doesn't support it)
        let location = properties.get("location").unwrap().as_object().unwrap();
        assert!(!location.contains_key("format"));
    }

    #[test]
    fn test_generate_bedrock_schema_weather_response() {
        let schema = generate_bedrock_schema::<aws_lambda_mcp::models::WeatherResponse>();
        let schema_value: Value = serde_json::from_value(schema).unwrap();

        // Check that required fields are present
        assert!(schema_value.get("type").is_some());
        assert!(schema_value.get("properties").is_some());

        // Check that all expected fields exist
        let properties = schema_value.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("location"));
        assert!(properties.contains_key("temperature"));
        assert!(properties.contains_key("temperature_unit"));
        assert!(properties.contains_key("weather_code"));
        assert!(properties.contains_key("wind_speed"));
    }

    #[test]
    fn test_write_schema() {
        let tools = vec![
            Tool {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: serde_json::json!({"type": "string"}),
            }
        ];

        write_schema(&tools);

        // Check that file was created
        assert!(std::path::Path::new("tool_schema.json").exists());

        // Check content
        let content = fs::read_to_string("tool_schema.json").unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 1);

        let tool = &array[0];
        assert_eq!(tool["name"], "test_tool");
        assert_eq!(tool["description"], "A test tool");

        // Clean up
        fs::remove_file("tool_schema.json").unwrap();
    }

    #[test]
    fn test_main_generates_valid_schema() {
        // This test calls main() which writes to tool_schema.json
        main();

        // Check that file was created
        assert!(std::path::Path::new("tool_schema.json").exists());

        // Check content is valid JSON
        let content = fs::read_to_string("tool_schema.json").unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert!(!array.is_empty());

        // Check that weather tool is present
        let weather_tool = array.iter().find(|t| t["name"] == "get_weather").unwrap();
        assert!(weather_tool["description"].as_str().unwrap().contains("weather"));
        assert!(weather_tool["inputSchema"].is_object());
        assert!(weather_tool["outputSchema"].is_object());

        // Clean up
        fs::remove_file("tool_schema.json").unwrap();
    }
}
