#![forbid(unsafe_code)]

use crate::catalog::{example_values, is_secret_env, Catalog, FlagSpec};
use serde_json::{json, Map, Value};

const DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";

/// JSON Schema 2020-12 for the *resolved* env map (catalog keys only).
///
/// Process environments contain many extra keys, so this schema is for the
/// overlay output (`load_env_map`), not `std::env::vars()` wholesale.
/// `additionalProperties` is false: unknown catalog keys are a contract break.
pub fn os_schema(catalog: &Catalog) -> Value {
    let mut properties = Map::new();
    let mut required = Vec::new();
    for flag in &catalog.flags {
        properties.insert(flag.env.clone(), os_property(flag));
        if flag.required && flag.default.is_none() {
            required.push(Value::String(flag.env.clone()));
        }
    }
    let mut schema = Map::new();
    schema.insert("$schema".into(), json!(DRAFT));
    schema.insert("$id".into(), json!(schema_id(catalog, "env.os.schema.json")));
    schema.insert(
        "title".into(),
        json!(format!("{} resolved environment", catalog.type_name)),
    );
    schema.insert(
        "description".into(),
        json!(os_description(catalog)),
    );
    schema.insert("type".into(), json!("object"));
    schema.insert("additionalProperties".into(), json!(false));
    schema.insert("properties".into(), Value::Object(properties));
    if !required.is_empty() {
        schema.insert("required".into(), Value::Array(required));
    }
    schema.insert("x-flags-2-env".into(), meta(catalog));
    Value::Object(schema)
}

/// JSON Schema 2020-12 for parsed typed values (`CliEnvValues`).
pub fn values_schema(catalog: &Catalog) -> Value {
    let mut properties = Map::new();
    let mut required = Vec::new();
    for flag in &catalog.flags {
        properties.insert(flag.snake.clone(), values_property(flag));
        if flag.default.is_some() || flag.required {
            required.push(Value::String(flag.snake.clone()));
        }
    }
    let mut schema = Map::new();
    schema.insert("$schema".into(), json!(DRAFT));
    schema.insert(
        "$id".into(),
        json!(schema_id(catalog, "env.values.schema.json")),
    );
    schema.insert(
        "title".into(),
        json!(format!("{}Values", catalog.type_name)),
    );
    schema.insert(
        "description".into(),
        json!(format!(
            "Parsed flags-2-env values for {}. extra properties are forbidden.",
            catalog.type_name
        )),
    );
    schema.insert("type".into(), json!("object"));
    schema.insert("additionalProperties".into(), json!(false));
    schema.insert("properties".into(), Value::Object(properties));
    if !required.is_empty() {
        schema.insert("required".into(), Value::Array(required));
    }
    schema.insert("x-flags-2-env".into(), meta(catalog));
    Value::Object(schema)
}

pub fn render_os_schema(catalog: &Catalog) -> String {
    pretty(os_schema(catalog))
}

pub fn render_values_schema(catalog: &Catalog) -> String {
    pretty(values_schema(catalog))
}

fn pretty(value: Value) -> String {
    let mut out = serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into());
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn schema_id(catalog: &Catalog, file: &str) -> String {
    let service = catalog
        .service
        .as_deref()
        .unwrap_or("service")
        .replace(' ', "-");
    format!("https://github.com/flags-2-env/flags-2-env-cli/generated/json-schema/{service}/{file}")
}

fn meta(catalog: &Catalog) -> Value {
    json!({
        "service": catalog.service,
        "typeName": catalog.type_name,
        "generator": "flags-2-env",
    })
}

fn os_description(catalog: &Catalog) -> String {
    format!(
        "Resolved environment map for {} after flags-2-env overlay \
(flags > env_shell > env_file unless reordered). Values are still strings, \
as the OS stores them. Validate this object at runtime with `check_os_env` \
or `f2e check-contract`; do not hand-edit generated sources.",
        catalog.type_name
    )
}

fn os_property(flag: &FlagSpec) -> Value {
    let mut prop = Map::new();
    prop.insert("type".into(), json!("string"));
    prop.insert("minLength".into(), json!(1));
    match flag.flag_type.as_str() {
        "bool" => {
            prop.insert(
                "enum".into(),
                json!(["0", "1", "true", "false", "TRUE", "FALSE", "yes", "no", "YES", "NO"]),
            );
        }
        "int" => {
            prop.insert("pattern".into(), json!(r"^-?[0-9]+$"));
        }
        "float" => {
            prop.insert(
                "pattern".into(),
                json!(r"^-?[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?$"),
            );
        }
        "json" | "map" | "array" => {
            prop.insert(
                "description".into(),
                json!("JSON-encoded value (object or array) as a string"),
            );
        }
        _ => {}
    }
    if let Some(help) = &flag.help {
        prop.insert("description".into(), json!(help));
    }
    if !is_secret_env(&flag.env) {
        let examples = example_values(flag);
        if !examples.is_empty() {
            prop.insert("examples".into(), json!(examples));
        }
    }
    prop.insert("x-env-key".into(), json!(flag.env));
    prop.insert("x-flag-type".into(), json!(flag.flag_type));
    Value::Object(prop)
}

fn values_property(flag: &FlagSpec) -> Value {
    let typed = match flag.flag_type.as_str() {
        "bool" => json!({"type": "boolean"}),
        "int" => json!({"type": "integer"}),
        "float" => json!({"type": "number"}),
        "array" => json!({"type": "array"}),
        "map" | "json" => json!({"type": "object"}),
        _ => json!({"type": "string", "minLength": 1}),
    };
    if flag.default.is_some() {
        return typed;
    }
    json!({ "anyOf": [typed, { "type": "null" }] })
}

#[cfg(test)]
mod tests {
    use super::{os_schema, render_os_schema, values_schema};
    use crate::catalog::parse_catalog;

    fn demo() -> crate::catalog::Catalog {
        parse_catalog(
            r#"
[identity]
service = "demo"
type_name = "DemoEnv"

[flags.bind]
env = "DEMO_BIND"
default = "127.0.0.1:8080"

[flags.token]
env = "DEMO_TOKEN"
required = true

[flags.workers]
env = "DEMO_WORKERS"
type = "int"
default = "2"

[flags.verbose]
env = "DEMO_VERBOSE"
type = "bool"
"#,
            None,
        )
        .unwrap()
    }

    #[test]
    fn os_schema_is_draft_2020_12_and_lists_required_keys() {
        let schema = os_schema(&demo());
        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["additionalProperties"], false);
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "DEMO_TOKEN"));
        assert_eq!(schema["properties"]["DEMO_WORKERS"]["pattern"], r"^-?[0-9]+$");
        assert!(schema["properties"]["DEMO_VERBOSE"]["enum"].is_array());
        let rendered = render_os_schema(&demo());
        assert!(rendered.contains("\"$schema\""));
        assert!(values_schema(&demo())["properties"]["workers"]["type"] == "integer");
    }
}
