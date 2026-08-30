#![forbid(unsafe_code)]

use crate::catalog::Catalog;
use crate::error::CliError;
use crate::generate::{os_schema, values_schema};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractFailure {
    pub path: String,
    pub message: String,
}

impl std::fmt::Display for ContractFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

/// Validate a resolved env map with JSON Schema 2020-12 (`jsonschema` crate).
pub fn check_os_map(
    catalog: &Catalog,
    env: &BTreeMap<String, String>,
) -> Result<(), Vec<ContractFailure>> {
    let instance = map_to_json(env);
    validate_instance(&os_schema(catalog), &instance, "env.os.schema.json")
}

pub fn check_values_json(catalog: &Catalog, instance: &Value) -> Result<(), Vec<ContractFailure>> {
    validate_instance(&values_schema(catalog), instance, "env.values.schema.json")
}

fn validate_instance(
    schema: &Value,
    instance: &Value,
    name: &str,
) -> Result<(), Vec<ContractFailure>> {
    let validator = jsonschema::validator_for(schema).map_err(|err| {
        vec![ContractFailure {
            path: name.into(),
            message: format!("schema failed to compile: {err}"),
        }]
    })?;
    let errors: Vec<ContractFailure> = validator
        .iter_errors(instance)
        .map(|err| ContractFailure {
            path: err.instance_path.to_string(),
            message: err.to_string(),
        })
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn map_to_json(env: &BTreeMap<String, String>) -> Value {
    Value::Object(
        env.iter()
            .map(|(key, value)| (key.clone(), Value::String(value.clone())))
            .collect(),
    )
}

pub fn failures_to_error(failures: Vec<ContractFailure>) -> CliError {
    let body = failures
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n  ");
    CliError::Command(format!("env contract violated:\n  {body}"))
}

#[cfg(test)]
mod tests {
    use super::{check_os_map, check_values_json};
    use crate::catalog::parse_catalog;
    use serde_json::json;
    use std::collections::BTreeMap;

    fn catalog() -> crate::catalog::Catalog {
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
"#,
            None,
        )
        .unwrap()
    }

    #[test]
    fn json_schema_accepts_a_valid_resolved_map() {
        let mut env = BTreeMap::new();
        env.insert("DEMO_BIND".into(), "127.0.0.1:9".into());
        env.insert("DEMO_TOKEN".into(), "tok_test".into());
        env.insert("DEMO_WORKERS".into(), "4".into());
        check_os_map(&catalog(), &env).expect("valid map");
    }

    #[test]
    fn json_schema_rejects_missing_required_and_bad_types_and_extras() {
        let cat = catalog();
        let mut missing = BTreeMap::new();
        missing.insert("DEMO_BIND".into(), "127.0.0.1:9".into());
        assert!(check_os_map(&cat, &missing).is_err());

        let mut bad_int = BTreeMap::new();
        bad_int.insert("DEMO_TOKEN".into(), "tok".into());
        bad_int.insert("DEMO_WORKERS".into(), "nope".into());
        assert!(check_os_map(&cat, &bad_int).is_err());

        let mut extra = BTreeMap::new();
        extra.insert("DEMO_TOKEN".into(), "tok".into());
        extra.insert("NOT_IN_CATALOG".into(), "x".into());
        assert!(check_os_map(&cat, &extra).is_err());
    }

    #[test]
    fn values_schema_rejects_wrong_runtime_types() {
        let cat = catalog();
        let ok = json!({
            "bind": "127.0.0.1:8080",
            "token": "tok",
            "workers": 2
        });
        check_values_json(&cat, &ok).expect("typed values");
        let bad = json!({
            "bind": "127.0.0.1:8080",
            "token": "tok",
            "workers": "two"
        });
        assert!(check_values_json(&cat, &bad).is_err());
    }
}
