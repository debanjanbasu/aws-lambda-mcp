use aws_lambda_mcp::models::{WeatherRequest, WeatherResponse};
use aws_lambda_mcp::tools::weather;
use schemars::{JsonSchema, schema_for};
use serde_json::{Value, json};
use std::fs;

struct Tool {
    name: String,
    description: String,
    input_schema: Value,
    output_schema: Value,
}

fn main() {
    let get_weather_attr = weather::get_weather_tool_attr();

    eprintln!("DEBUG: Tool name: {}", get_weather_attr.name);
    eprintln!(
        "DEBUG: Tool description: {:?}",
        get_weather_attr.description
    );

    let tools = vec![Tool {
        name: get_weather_attr.name.to_string(),
        description: get_weather_attr
            .description
            .map_or_else(String::new, String::from),
        input_schema: generate_bedrock_schema::<WeatherRequest>(),
        output_schema: generate_bedrock_schema::<WeatherResponse>(),
    }];

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
