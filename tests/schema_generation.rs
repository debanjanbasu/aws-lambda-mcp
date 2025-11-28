//! Tests for schema generation functionality

use std::fs;
use std::process::Command;

#[test]
fn test_schema_generation_produces_valid_json() {
    // Run the schema generation binary
    let output = Command::new("cargo")
        .args(["run", "--bin", "generate-schema", "--features", "schema-gen"])
        .output()
        .expect("Failed to run schema generation");

    assert!(output.status.success(), "Schema generation failed: {:?}", output);

    // Check that tool_schema.json was created
    assert!(fs::metadata("tool_schema.json").is_ok(), "tool_schema.json was not created");

    // Read and parse the JSON
    let content = fs::read_to_string("tool_schema.json").expect("Failed to read tool_schema.json");
    let schema: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON in tool_schema.json");

    // Verify it's an array
    assert!(schema.is_array(), "Schema should be an array");

    let tools = schema.as_array().unwrap();
    assert!(!tools.is_empty(), "Schema should contain at least one tool");

    // Verify each tool has required fields
    for tool in tools {
        assert!(tool.is_object(), "Each tool should be an object");
        let tool_obj = tool.as_object().unwrap();

        // Check required fields
        assert!(tool_obj.contains_key("name"), "Tool missing 'name' field");
        assert!(tool_obj.contains_key("description"), "Tool missing 'description' field");
        assert!(tool_obj.contains_key("inputSchema"), "Tool missing 'inputSchema' field");
        assert!(tool_obj.contains_key("outputSchema"), "Tool missing 'outputSchema' field");

        // Verify name and description are strings
        assert!(tool_obj["name"].is_string(), "Tool name should be a string");
        assert!(tool_obj["description"].is_string(), "Tool description should be a string");

        // Verify schemas are objects
        assert!(tool_obj["inputSchema"].is_object(), "inputSchema should be an object");
        assert!(tool_obj["outputSchema"].is_object(), "outputSchema should be an object");
    }

    // Clean up
    let _ = fs::remove_file("tool_schema.json");
}

#[test]
fn test_weather_tool_schema_structure() {
    // Run schema generation
    let output = Command::new("cargo")
        .args(["run", "--bin", "generate-schema", "--features", "schema-gen"])
        .output()
        .expect("Failed to run schema generation");

    assert!(output.status.success(), "Schema generation failed");

    // Read the schema
    let content = fs::read_to_string("tool_schema.json").expect("Failed to read schema");
    let schema: Vec<serde_json::Value> = serde_json::from_str(&content).expect("Invalid JSON");

    // Find the weather tool
    let weather_tool = schema.iter().find(|tool| tool["name"] == "get_weather")
        .expect("Weather tool not found in schema");

    // Verify weather tool structure
    assert_eq!(weather_tool["name"], "get_weather");
    assert!(weather_tool["description"].as_str().unwrap().contains("weather"));

    // Check input schema
    let input_schema = &weather_tool["inputSchema"];
    assert_eq!(input_schema["type"], "object");
    assert!(input_schema["properties"].is_object());

    let properties = input_schema["properties"].as_object().unwrap();
    assert!(properties.contains_key("location"), "Weather tool should have location input");

    let location_prop = &properties["location"];
    assert_eq!(location_prop["type"], "string");
    assert!(location_prop["description"].is_string());

    // Check required fields
    let required = input_schema["required"].as_array().unwrap();
    assert!(required.contains(&serde_json::json!("location")));

    // Check output schema
    let output_schema = &weather_tool["outputSchema"];
    assert_eq!(output_schema["type"], "object");
    assert!(output_schema["properties"].is_object());

    let output_props = output_schema["properties"].as_object().unwrap();
    let expected_outputs = ["location", "temperature", "temperature_unit", "weather_code", "wind_speed"];
    for field in &expected_outputs {
        assert!(output_props.contains_key(*field), "Missing output field: {}", field);
    }

    // Clean up
    let _ = fs::remove_file("tool_schema.json");
}