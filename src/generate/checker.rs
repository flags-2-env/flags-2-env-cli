#![forbid(unsafe_code)]

use crate::catalog::{example_values, Catalog, FlagType};
use crate::generate::json_schema::{os_schema, render_os_schema, render_values_schema};

/// Dependency-free contract checker emitted into generated runtimes.
pub fn render_rust(catalog: &Catalog) -> String {
    let mut out = String::from(
        r#"
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractError {
    pub path: String,
    pub message: String,
}

impl std::fmt::Display for ContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

/// Validate the resolved env map against the generated JSON Schema rules.
/// Call this at runtime (after overlay), not only at compile time.
pub fn check_os_env(
    env: &std::collections::BTreeMap<String, String>,
) -> Result<(), Vec<ContractError>> {
    let mut errors = Vec::new();
"#,
    );
    for flag in &catalog.flags {
        let env = rust_escape(&flag.env);
        let check = rust_type_check_fn(&flag.flag_type);
        if flag.required && flag.default.is_none() {
            out.push_str(&format!(
                "    match env.get(\"{env}\").map(|value| value.trim()).filter(|value| !value.is_empty()) {{\n        None => errors.push(ContractError {{ path: \"{env}\".into(), message: \"missing required environment variable\".into() }}),\n        Some(raw) => {{\n            if let Some(message) = {check}(raw) {{\n                errors.push(ContractError {{ path: \"{env}\".into(), message }});\n            }}\n        }}\n    }}\n"
            ));
        } else {
            out.push_str(&format!(
                "    if let Some(raw) = env.get(\"{env}\").map(|value| value.trim()).filter(|value| !value.is_empty()) {{\n        if let Some(message) = {check}(raw) {{\n            errors.push(ContractError {{ path: \"{env}\".into(), message }});\n        }}\n    }}\n"
            ));
        }
    }
    out.push_str(
        r#"    for key in env.keys() {
        if !KNOWN_ENV_KEYS.contains(&key.as_str()) {
            errors.push(ContractError {
                path: key.clone(),
                message: "additional property not in the env contract".into(),
            });
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn contract_error_to_missing(error: &ContractError) -> MissingEnv {
    match error.path.as_str() {
"#,
    );
    for flag in &catalog.flags {
        let examples = rust_examples(flag);
        out.push_str(&format!(
            "        \"{env}\" => MissingEnv {{ name: \"{env}\", expected_type: \"{ty}\", examples: {examples} }},\n",
            env = rust_escape(&flag.env),
            ty = rust_escape(flag.flag_type.as_str()),
            examples = examples,
        ));
    }
    out.push_str(
        r#"        _ => MissingEnv {
            name: "ENV_CONTRACT",
            expected_type: "json-schema-2020-12",
            examples: &[],
        },
    }
}

pub fn assert_os_env(env: &std::collections::BTreeMap<String, String>) {
    if let Err(errors) = check_os_env(env) {
        panic!(
            "environment contract violated: {}",
            errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ")
        );
    }
}

"#,
    );
    out.push_str("const KNOWN_ENV_KEYS: &[&str] = &[");
    for flag in &catalog.flags {
        out.push_str(&format!("\"{}\", ", rust_escape(&flag.env)));
    }
    out.push_str("];\n\n");
    out.push_str(RUST_TYPE_CHECKS);
    out.push_str(&format!(
        "pub const OS_ENV_SCHEMA_JSON: &str = {};\n",
        rust_string_literal(&render_os_schema(catalog))
    ));
    out.push_str(&format!(
        "pub const VALUES_SCHEMA_JSON: &str = {};\n",
        rust_string_literal(&render_values_schema(catalog))
    ));
    out
}

pub fn render_typescript(catalog: &Catalog) -> String {
    let mut out = String::from(
        r#"
export interface ContractError {
  readonly path: string;
  readonly message: string;
}

/** Validate the resolved env map against the generated JSON Schema rules. */
export function checkOsEnv(env: Record<string, string>): ContractError[] {
  const errors: ContractError[] = [];
"#,
    );
    for flag in &catalog.flags {
        let key = format!("\"{}\"", ts_escape(&flag.env));
        let check = ts_type_check_fn(&flag.flag_type);
        if flag.required && flag.default.is_none() {
            out.push_str(&format!(
                "  {{\n    const raw = nonEmpty(env[{key}]);\n    if (raw === undefined) errors.push({{ path: {key}, message: \"missing required environment variable\" }});\n    else {{\n      const message = {check}(raw);\n      if (message) errors.push({{ path: {key}, message }});\n    }}\n  }}\n"
            ));
        } else {
            out.push_str(&format!(
                "  {{\n    const raw = nonEmpty(env[{key}]);\n    if (raw !== undefined) {{\n      const message = {check}(raw);\n      if (message) errors.push({{ path: {key}, message }});\n    }}\n  }}\n"
            ));
        }
    }
    out.push_str(
        "  for (const key of Object.keys(env)) {\n    if (!KNOWN_ENV_KEYS.includes(key)) {\n      errors.push({ path: key, message: \"additional property not in the env contract\" });\n    }\n  }\n  return errors;\n}\n\n",
    );
    out.push_str(
        "export function assertOsEnv(env: Record<string, string>): void {\n  const errors = checkOsEnv(env);\n  if (errors.length > 0) {\n    throw new Error(`environment contract violated: ${errors.map((error) => `${error.path}: ${error.message}`).join(\"; \")}`);\n  }\n}\n\n",
    );
    out.push_str("const KNOWN_ENV_KEYS: readonly string[] = [");
    for flag in &catalog.flags {
        out.push_str(&format!("\"{}\", ", ts_escape(&flag.env)));
    }
    out.push_str("];\n");
    out.push_str(TS_TYPE_CHECKS);
    out.push_str(&format!(
        "export const OS_ENV_SCHEMA = {} as const;\n",
        serde_json::to_string(&os_schema(catalog)).unwrap_or_else(|_| "{}".into())
    ));
    out
}

pub fn render_dart(catalog: &Catalog) -> String {
    let mut out = String::from(
        r#"
class ContractError {
  const ContractError({required this.path, required this.message});
  final String path;
  final String message;
  @override
  String toString() => '$path: $message';
}

/// Validate the resolved env map against the generated JSON Schema rules.
List<ContractError> checkOsEnv(Map<String, String> env) {
  final errors = <ContractError>[];
"#,
    );
    for flag in &catalog.flags {
        let key = format!("'{}'", dart_escape(&flag.env));
        let check = dart_type_check_fn(&flag.flag_type);
        if flag.required && flag.default.is_none() {
            out.push_str(&format!(
                "  {{\n    final raw = _nonEmpty(env[{key}]);\n    if (raw == null) {{\n      errors.add(const ContractError(path: {key}, message: 'missing required environment variable'));\n    }} else {{\n      final message = {check}(raw);\n      if (message != null) errors.add(ContractError(path: {key}, message: message));\n    }}\n  }}\n"
            ));
        } else {
            out.push_str(&format!(
                "  {{\n    final raw = _nonEmpty(env[{key}]);\n    if (raw != null) {{\n      final message = {check}(raw);\n      if (message != null) errors.add(ContractError(path: {key}, message: message));\n    }}\n  }}\n"
            ));
        }
    }
    out.push_str(
        "  for (final key in env.keys) {\n    if (!_knownEnvKeys.contains(key)) {\n      errors.add(ContractError(path: key, message: 'additional property not in the env contract'));\n    }\n  }\n  return errors;\n}\n\n",
    );
    out.push_str(
        "void assertOsEnv(Map<String, String> env) {\n  final errors = checkOsEnv(env);\n  if (errors.isNotEmpty) {\n    throw StateError('environment contract violated: ${errors.join('; ')}');\n  }\n}\n\n",
    );
    out.push_str("const _knownEnvKeys = <String>{");
    for flag in &catalog.flags {
        out.push_str(&format!("'{}', ", dart_escape(&flag.env)));
    }
    out.push_str("};\n");
    out.push_str(&render_dart_type_checks(catalog));
    out
}

fn render_dart_type_checks(catalog: &Catalog) -> String {
    let mut out = String::new();
    let has = |expected| catalog.flags.iter().any(|flag| flag.flag_type == expected);

    if has(FlagType::String) {
        out.push_str(DART_STRING_CHECK);
    }
    if has(FlagType::Bool) {
        out.push_str(DART_BOOL_CHECK);
    }
    if has(FlagType::Int) {
        out.push_str(DART_INT_CHECK);
    }
    if has(FlagType::Float) {
        out.push_str(DART_FLOAT_CHECK);
    }
    if has(FlagType::Array) || has(FlagType::Map) || has(FlagType::Json) {
        out.push_str(DART_JSON_CHECK);
    }
    out
}

fn rust_examples(flag: &crate::catalog::FlagSpec) -> String {
    format!(
        "&[{}]",
        example_values(flag)
            .iter()
            .map(|value| format!("\"{}\"", rust_escape(value)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn rust_type_check_fn(flag_type: &FlagType) -> &'static str {
    match flag_type {
        FlagType::Bool => "contract_check_bool",
        FlagType::Int => "contract_check_int",
        FlagType::Float => "contract_check_float",
        FlagType::Json | FlagType::Map | FlagType::Array => "contract_check_json",
        FlagType::String => "contract_check_string",
    }
}

fn ts_type_check_fn(flag_type: &FlagType) -> &'static str {
    match flag_type {
        FlagType::Bool => "contractCheckBool",
        FlagType::Int => "contractCheckInt",
        FlagType::Float => "contractCheckFloat",
        FlagType::Json | FlagType::Map | FlagType::Array => "contractCheckJson",
        FlagType::String => "contractCheckString",
    }
}

fn dart_type_check_fn(flag_type: &FlagType) -> &'static str {
    match flag_type {
        FlagType::Bool => "_contractCheckBool",
        FlagType::Int => "_contractCheckInt",
        FlagType::Float => "_contractCheckFloat",
        FlagType::Json | FlagType::Map | FlagType::Array => "_contractCheckJson",
        FlagType::String => "_contractCheckString",
    }
}

fn rust_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn ts_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn dart_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn rust_string_literal(value: &str) -> String {
    format!("{:?}", value)
}

const RUST_TYPE_CHECKS: &str = r#"
fn contract_check_string(raw: &str) -> Option<String> {
    if raw.is_empty() {
        Some("empty string".into())
    } else {
        None
    }
}

fn contract_check_bool(raw: &str) -> Option<String> {
    match raw {
        "0" | "1" | "true" | "false" | "TRUE" | "FALSE" | "yes" | "no" | "YES" | "NO" => None,
        _ => Some(format!("not a bool env token: {raw}")),
    }
}

fn contract_check_int(raw: &str) -> Option<String> {
    raw.parse::<i64>().err().map(|err| err.to_string())
}

fn contract_check_float(raw: &str) -> Option<String> {
    raw.parse::<f64>().err().map(|err| err.to_string())
}

fn contract_check_json(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        None
    } else {
        Some("expected JSON object or array string".into())
    }
}

"#;

const TS_TYPE_CHECKS: &str = r#"
function contractCheckString(raw: string): string | undefined {
  return raw.length === 0 ? "empty string" : undefined;
}
function contractCheckBool(raw: string): string | undefined {
  switch (raw) {
    case "0":
    case "1":
    case "true":
    case "false":
    case "TRUE":
    case "FALSE":
    case "yes":
    case "no":
    case "YES":
    case "NO":
      return undefined;
    default:
      return `not a bool env token: ${raw}`;
  }
}
function contractCheckInt(raw: string): string | undefined {
  return /^-?[0-9]+$/.test(raw) ? undefined : `not an int: ${raw}`;
}
function contractCheckFloat(raw: string): string | undefined {
  return Number.isNaN(Number.parseFloat(raw)) ? `not a float: ${raw}` : undefined;
}
function contractCheckJson(raw: string): string | undefined {
  try {
    const parsed = JSON.parse(raw) as unknown;
    if (parsed && typeof parsed === "object") return undefined;
    return "expected JSON object or array string";
  } catch {
    return "expected JSON object or array string";
  }
}

"#;

const DART_STRING_CHECK: &str = r#"
String? _contractCheckString(String raw) => raw.isEmpty ? 'empty string' : null;
"#;

const DART_BOOL_CHECK: &str = r#"
String? _contractCheckBool(String raw) {
  switch (raw) {
    case '0':
    case '1':
    case 'true':
    case 'false':
    case 'TRUE':
    case 'FALSE':
    case 'yes':
    case 'no':
    case 'YES':
    case 'NO':
      return null;
    default:
      return 'not a bool env token: $raw';
  }
}
"#;

const DART_INT_CHECK: &str = r#"
String? _contractCheckInt(String raw) => int.tryParse(raw) == null ? 'not an int: $raw' : null;
"#;

const DART_FLOAT_CHECK: &str = r#"
String? _contractCheckFloat(String raw) => double.tryParse(raw) == null ? 'not a float: $raw' : null;
"#;

const DART_JSON_CHECK: &str = r#"
String? _contractCheckJson(String raw) {
  final trimmed = raw.trim();
  final ok = (trimmed.startsWith('{') && trimmed.endsWith('}')) ||
      (trimmed.startsWith('[') && trimmed.endsWith(']'));
  return ok ? null : 'expected JSON object or array string';
}

"#;

#[cfg(test)]
mod tests {
    use super::render_rust;
    use crate::catalog::parse_catalog;

    #[test]
    fn rust_checker_mentions_json_schema_and_required_key() {
        let catalog = parse_catalog(
            r#"
[flags.token]
env = "API_TOKEN"
required = true
[flags.port]
env = "APP_PORT"
type = "int"
default = "8080"
"#,
            Some("CliEnv"),
        )
        .unwrap();
        let source = render_rust(&catalog);
        assert!(source.contains("pub fn check_os_env"));
        assert!(source.contains("pub fn assert_os_env"));
        assert!(source.contains("OS_ENV_SCHEMA_JSON"));
        assert!(source.contains("API_TOKEN"));
        assert!(source.contains("contract_check_int"));
        assert!(source.contains("json-schema"));
    }
}
