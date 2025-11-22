use std::fs;
use std::process::Command;

#[test]
fn test_generate_schema_produces_valid_json() {
    // Run the generate-schema binary
    let output = Command::new("cargo")
        .args(&["run", "--bin", "generate-schema", "--features", "schema-gen"])
        .output()
        .expect("Failed to run generate-schema");

    assert!(output.status.success(), "generate-schema failed: {:?}", output);

    // Check that tool_schema.json was created
    assert!(fs::metadata("tool_schema.json").is_ok(), "tool_schema.json not created");

    // Read and parse the JSON
    let content = fs::read_to_string("tool_schema.json").expect("Failed to read tool_schema.json");
    let json: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON in tool_schema.json");

    // Verify it's an array
    assert!(json.is_array(), "tool_schema.json should be an array");

    let tools = json.as_array().unwrap();
    assert!(!tools.is_empty(), "tool_schema.json should not be empty");

    // Check structure of first tool
    if let Some(tool) = tools.first() {
        assert!(tool.is_object(), "Each tool should be an object");
        let tool_obj = tool.as_object().unwrap();
        assert!(tool_obj.contains_key("name"), "Tool should have name");
        assert!(tool_obj.contains_key("description"), "Tool should have description");
        assert!(tool_obj.contains_key("inputSchema"), "Tool should have inputSchema");
        assert!(tool_obj.contains_key("outputSchema"), "Tool should have outputSchema");
    }

    // Clean up
    let _ = fs::remove_file("tool_schema.json");
}