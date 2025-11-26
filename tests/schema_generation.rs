//! Integration tests for schema generation functionality

use std::fs;
use std::process::Command;

#[test]
fn test_generate_schema_creates_file() {
    // Clean up any existing file
    let _ = fs::remove_file("tool_schema.json");

    // Run the generate-schema binary
    let output = Command::new("cargo")
        .args(["run", "--bin", "generate-schema", "--features", "schema-gen"])
        .output()
        .expect("Failed to run generate-schema");

    assert!(output.status.success(), "generate-schema failed: {:?}", output);

    // Check that the file was created
    assert!(fs::metadata("tool_schema.json").is_ok(), "tool_schema.json was not created");

    // Clean up
    let _ = fs::remove_file("tool_schema.json");
}

#[test]
fn test_generate_schema_output_format() {
    // Clean up any existing file
    let _ = fs::remove_file("tool_schema.json");

    // Run the generate-schema binary
    let output = Command::new("cargo")
        .args(["run", "--bin", "generate-schema", "--features", "schema-gen"])
        .output()
        .expect("Failed to run generate-schema");

    assert!(output.status.success(), "generate-schema failed: {:?}", output);

    // Read and parse the generated JSON
    let content = fs::read_to_string("tool_schema.json")
        .expect("Failed to read tool_schema.json");

    let schemas: serde_json::Value = serde_json::from_str(&content)
        .expect("Failed to parse tool_schema.json as JSON");

    // Verify it's an array
    assert!(schemas.is_array(), "Schema should be an array");

    let schemas_array = schemas.as_array().unwrap();
    assert!(!schemas_array.is_empty(), "Schema array should not be empty");

    // Check the structure of the first tool
    let first_tool = &schemas_array[0];
    assert!(first_tool.is_object(), "Each tool should be an object");

    let tool_obj = first_tool.as_object().unwrap();

    // Required fields
    assert!(tool_obj.contains_key("name"), "Tool should have 'name' field");
    assert!(tool_obj.contains_key("description"), "Tool should have 'description' field");
    assert!(tool_obj.contains_key("inputSchema"), "Tool should have 'inputSchema' field");
    assert!(tool_obj.contains_key("outputSchema"), "Tool should have 'outputSchema' field");

    // Check inputSchema structure
    let input_schema = tool_obj.get("inputSchema").unwrap();
    assert!(input_schema.is_object(), "inputSchema should be an object");
    let input_obj = input_schema.as_object().unwrap();
    assert!(input_obj.contains_key("type"), "inputSchema should have 'type' field");
    assert_eq!(input_obj.get("type").unwrap(), "object", "inputSchema type should be 'object'");

    // Check outputSchema structure
    let output_schema = tool_obj.get("outputSchema").unwrap();
    assert!(output_schema.is_object(), "outputSchema should be an object");
    let output_obj = output_schema.as_object().unwrap();
    assert!(output_obj.contains_key("type"), "outputSchema should have 'type' field");
    assert_eq!(output_obj.get("type").unwrap(), "object", "outputSchema type should be 'object'");

    // Clean up
    let _ = fs::remove_file("tool_schema.json");
}

#[test]
fn test_generate_schema_weather_tool() {
    // Clean up any existing file
    let _ = fs::remove_file("tool_schema.json");

    // Run the generate-schema binary
    let output = Command::new("cargo")
        .args(["run", "--bin", "generate-schema", "--features", "schema-gen"])
        .output()
        .expect("Failed to run generate-schema");

    assert!(output.status.success(), "generate-schema failed: {:?}", output);

    // Read and parse the generated JSON
    let content = fs::read_to_string("tool_schema.json")
        .expect("Failed to read tool_schema.json");

    let schemas: Vec<serde_json::Value> = serde_json::from_str(&content)
        .expect("Failed to parse tool_schema.json as JSON array");

    // Find the weather tool
    let weather_tool = schemas.iter()
        .find(|tool| tool.get("name").unwrap() == "get_weather")
        .expect("get_weather tool not found in schema");

    // Verify weather tool structure
    assert_eq!(weather_tool.get("name").unwrap(), "get_weather");

    let description = weather_tool.get("description").unwrap().as_str().unwrap();
    assert!(description.contains("weather"), "Description should mention weather");
    assert!(description.contains("temperature"), "Description should mention temperature");

    // Check input schema has location field
    let input_schema = weather_tool.get("inputSchema").unwrap();
    let properties = input_schema.get("properties").unwrap();
    assert!(properties.get("location").is_some(), "Input schema should have location field");

    let required = input_schema.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&serde_json::json!("location")), "location should be required");

    // Check output schema has expected fields
    let output_schema = weather_tool.get("outputSchema").unwrap();
    let output_properties = output_schema.get("properties").unwrap();
    let expected_fields = ["location", "temperature", "temperature_unit", "weather_code", "wind_speed"];

    for field in &expected_fields {
        assert!(output_properties.get(*field).is_some(), "Output schema should have {} field", field);
    }

    let output_required = output_schema.get("required").unwrap().as_array().unwrap();
    for field in &expected_fields {
        assert!(output_required.contains(&serde_json::json!(field)), "{} should be required in output", field);
    }

    // Clean up
    let _ = fs::remove_file("tool_schema.json");
}

#[test]
fn test_generate_schema_stdout_message() {
    // Clean up any existing file
    let _ = fs::remove_file("tool_schema.json");

    // Run the generate-schema binary
    let output = Command::new("cargo")
        .args(["run", "--bin", "generate-schema", "--features", "schema-gen"])
        .output()
        .expect("Failed to run generate-schema");

    assert!(output.status.success(), "generate-schema failed: {:?}", output);

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("✅ Generated tool_schema.json"), "Should print success message");
    assert!(stdout.contains("tool(s)"), "Should mention number of tools");

    // Clean up
    let _ = fs::remove_file("tool_schema.json");
}

#[test]
fn test_generate_schema_overwrites_existing_file() {
    // Create a dummy file first
    fs::write("tool_schema.json", "dummy content").expect("Failed to write dummy file");

    // Verify it exists with dummy content
    let initial_content = fs::read_to_string("tool_schema.json").unwrap();
    assert_eq!(initial_content, "dummy content");

    // Run the generate-schema binary
    let output = Command::new("cargo")
        .args(["run", "--bin", "generate-schema", "--features", "schema-gen"])
        .output()
        .expect("Failed to run generate-schema");

    assert!(output.status.success(), "generate-schema failed: {:?}", output);

    // Verify the file was overwritten with valid JSON
    let new_content = fs::read_to_string("tool_schema.json").unwrap();
    assert_ne!(new_content, "dummy content");

    // Should be valid JSON
    let _: serde_json::Value = serde_json::from_str(&new_content)
        .expect("Generated content should be valid JSON");

    // Clean up
    let _ = fs::remove_file("tool_schema.json");
}