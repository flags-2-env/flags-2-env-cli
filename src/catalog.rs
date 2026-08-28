#![forbid(unsafe_code)]

use crate::error::CliError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlagSpec {
    pub name: String,
    pub env: String,
    pub rust_const: String,
    pub snake: String,
    pub flag_type: String,
    pub default: Option<String>,
    pub help: Option<String>,
    pub required: bool,
    pub examples: Vec<String>,
    pub dotenv_override: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvLoad {
    pub load: bool,
    pub files: Vec<String>,
    pub override_dotenv: bool,
    pub order: Vec<String>,
}

impl Default for EnvLoad {
    fn default() -> Self {
        Self {
            load: false,
            files: Vec::new(),
            override_dotenv: false,
            order: default_source_order(),
        }
    }
}

pub fn default_source_order() -> Vec<String> {
    vec!["flags".into(), "env_shell".into(), "env_file".into()]
}

pub fn dotenv_override_order() -> Vec<String> {
    vec!["flags".into(), "env_file".into(), "env_shell".into()]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    pub service: Option<String>,
    pub type_name: String,
    pub flags: Vec<FlagSpec>,
    pub env_load: EnvLoad,
    pub order_of_preference: Vec<(String, Vec<String>)>,
}

pub fn parse_catalog(text: &str, type_name_override: Option<&str>) -> Result<Catalog, CliError> {
    let table: toml::Table = text
        .parse()
        .map_err(|err| CliError::Config(format!("invalid .cli-flags.toml: {err}")))?;
    let identity = table.get("identity").and_then(|value| value.as_table());
    let service = identity
        .and_then(|row| row.get("service"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let type_name = type_name_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            identity
                .and_then(|row| row.get("type_name"))
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "CliEnv".to_string());
    if !is_ident_start(&type_name) {
        return Err(CliError::Config(format!(
            "type name {type_name} is not a valid identifier"
        )));
    }
    let mut flags = Vec::new();
    collect_flags(&table, &mut flags)?;
    flags = uniquify_flags(flags);
    if flags.is_empty() {
        return Err(CliError::Config(
            "no [flags.*] entries with env keys found".into(),
        ));
    }
    Ok(Catalog {
        service,
        type_name,
        flags,
        env_load: parse_env_load(table.get("env").and_then(|value| value.as_table()))?,
        order_of_preference: parse_order_of_preference(
            table
                .get("order-of-preference")
                .and_then(|value| value.as_table()),
        ),
    })
}

fn parse_env_load(table: Option<&toml::Table>) -> Result<EnvLoad, CliError> {
    let Some(table) = table else {
        return Ok(EnvLoad::default());
    };
    let mut load = EnvLoad::default();
    let load_explicit = table.get("load").and_then(|value| value.as_bool());
    if let Some(files) = table.get("files") {
        let parsed = toml_string_list(files);
        for path in &parsed {
            if !is_safe_dotenv_path(path) {
                return Err(CliError::Config(format!(
                    "refusing dotenv path {path}: only relative .env* files are allowed"
                )));
            }
        }
        load.files = parsed;
    }
    if let Some(flag) = table.get("override").and_then(|value| value.as_bool()) {
        load.override_dotenv = flag;
    }
    if let Some(order) = table.get("order") {
        let parsed = toml_string_list(order);
        if parsed.len() >= 2 {
            load.order = complete_source_order(parsed);
        }
    }
    load.load = match load_explicit {
        Some(false) => false,
        Some(true) => true,
        None => !load.files.is_empty(),
    };
    if !load.load {
        load.files.clear();
    } else if load.files.is_empty() {
        load.files.push(".env".into());
    }
    Ok(load)
}

fn parse_order_of_preference(table: Option<&toml::Table>) -> Vec<(String, Vec<String>)> {
    let Some(table) = table else {
        return Vec::new();
    };
    table
        .iter()
        .map(|(key, value)| (key.clone(), complete_source_order(toml_string_list(value))))
        .filter(|(_, order)| order.len() >= 2)
        .collect()
}

fn complete_source_order(mut order: Vec<String>) -> Vec<String> {
    order.retain(|source| matches!(source.as_str(), "flags" | "env_shell" | "env_file"));
    for source in default_source_order() {
        if !order.iter().any(|item| item == &source) {
            order.push(source);
        }
    }
    order
}

fn collect_flags(table: &toml::Table, flags: &mut Vec<FlagSpec>) -> Result<(), CliError> {
    if let Some(flag_tables) = table.get("flags").and_then(|value| value.as_table()) {
        for (name, value) in flag_tables {
            let Some(flag_table) = value.as_table() else {
                continue;
            };
            if flag_table
                .get("env")
                .and_then(|value| value.as_str())
                .is_none()
            {
                continue;
            }
            flags.push(flag_from_table(name, flag_table)?);
        }
    }
    if let Some(commands) = table.get("commands").and_then(|value| value.as_table()) {
        for (_name, value) in commands {
            if let Some(command) = value.as_table() {
                collect_flags(command, flags)?;
            }
        }
    }
    Ok(())
}

fn uniquify_flags(flags: Vec<FlagSpec>) -> Vec<FlagSpec> {
    let mut seen_env = std::collections::BTreeSet::new();
    let mut seen_snake = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for mut flag in flags {
        if !seen_env.insert(flag.env.clone()) {
            continue;
        }
        if !seen_snake.insert(flag.snake.clone()) {
            let mut candidate = sanitize_field_ident(&to_snake(&flag.env));
            let mut n = 2u32;
            while !seen_snake.insert(candidate.clone()) {
                candidate = format!("{}_{n}", sanitize_field_ident(&to_snake(&flag.env)));
                n += 1;
            }
            flag.snake = candidate;
            flag.rust_const = to_snake(&flag.env).to_ascii_uppercase();
        }
        out.push(flag);
    }
    out
}

fn flag_from_table(name: &str, table: &toml::Table) -> Result<FlagSpec, CliError> {
    let env = table
        .get("env")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| CliError::Config(format!("flag {name} is missing env")))?
        .to_string();
    if !is_env_key(&env) {
        return Err(CliError::Config(format!(
            "flag {name} env {env} is not a valid ENV key (use A-Z, digits, underscore)"
        )));
    }
    let raw_snake = to_snake(name);
    let rust_const = raw_snake.to_ascii_uppercase();
    let snake = sanitize_field_ident(&raw_snake);
    let flag_type = table
        .get("type")
        .and_then(|value| value.as_str())
        .map(normalize_type)
        .or_else(|| {
            table
                .get("switch")
                .and_then(|value| value.as_bool())
                .filter(|value| *value)
                .map(|_| "bool".to_string())
        })
        .unwrap_or_else(|| "string".to_string());
    let default = table.get("default").and_then(toml_to_string);
    let help = table
        .get("help")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let required = table
        .get("required")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let examples = table
        .get("examples")
        .or_else(|| table.get("example"))
        .map(toml_string_list)
        .unwrap_or_default();
    let dotenv_override = table
        .get("dotenv_override")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    Ok(FlagSpec {
        name: name.to_string(),
        env,
        rust_const,
        snake,
        flag_type,
        default,
        help,
        required,
        examples,
        dotenv_override,
    })
}

fn toml_string_list(value: &toml::Value) -> Vec<String> {
    match value {
        toml::Value::String(text) => vec![text.clone()],
        toml::Value::Array(items) => items.iter().filter_map(toml_to_string).collect(),
        _ => Vec::new(),
    }
}

fn toml_to_string(value: &toml::Value) -> Option<String> {
    match value {
        toml::Value::String(text) => Some(text.clone()),
        toml::Value::Boolean(flag) => Some(flag.to_string()),
        toml::Value::Integer(number) => Some(number.to_string()),
        toml::Value::Float(number) => Some(number.to_string()),
        _ => None,
    }
}

fn normalize_type(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "bool" | "boolean" => "bool".into(),
        "int" | "integer" => "int".into(),
        "float" | "number" => "float".into(),
        "array" => "array".into(),
        "map" | "object" => "map".into(),
        "json" => "json".into(),
        _ => "string".into(),
    }
}

pub fn to_snake(name: &str) -> String {
    let mut out = String::new();
    let mut previous_lower = false;
    for ch in name.chars() {
        if ch == '-' || ch == '_' || ch == '.' || ch == ' ' {
            if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
            previous_lower = false;
            continue;
        }
        if !ch.is_ascii_alphanumeric() {
            continue;
        }
        if ch.is_ascii_uppercase() && previous_lower && !out.ends_with('_') {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
        previous_lower = ch.is_ascii_lowercase();
    }
    let trimmed = out.trim_matches('_').to_string();
    if trimmed.is_empty() {
        "value".into()
    } else if trimmed.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        format!("n_{trimmed}")
    } else {
        trimmed
    }
}

pub fn to_pascal(name: &str) -> String {
    to_snake(name)
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

impl Catalog {
    pub fn source_order_for(&self, flag: &FlagSpec) -> Vec<String> {
        if let Some((_, order)) = self
            .order_of_preference
            .iter()
            .find(|(key, _)| key == &flag.env)
        {
            return order.clone();
        }
        if flag.dotenv_override || self.env_load.override_dotenv {
            return dotenv_override_order();
        }
        self.env_load.order.clone()
    }
}

pub fn example_values(flag: &FlagSpec) -> Vec<String> {
    if is_secret_env(&flag.env) {
        return vec!["<redacted>".into()];
    }
    if !flag.examples.is_empty() {
        return flag.examples.clone();
    }
    match flag.flag_type.as_str() {
        "bool" => vec!["true".into(), "false".into(), "1".into(), "0".into()],
        "int" => vec!["8080".into(), "0".into()],
        "float" => vec!["1.0".into(), "0.5".into()],
        _ if flag.env.contains("URL")
            || flag.env.contains("BASE")
            || flag.env.contains("ORIGIN") =>
        {
            vec![
                "http://127.0.0.1:8080".into(),
                "https://api.example.test".into(),
            ]
        }
        _ if flag.env.contains("BIND") || flag.env.contains("LISTEN") => {
            vec!["127.0.0.1:8080".into(), "0.0.0.0:8080".into()]
        }
        _ => vec!["example-value".into()],
    }
}

pub fn is_secret_env(env: &str) -> bool {
    let upper = env.to_ascii_uppercase();
    if upper.contains("PRIVATE_KEY") || upper.contains("ACCESS_KEY") || upper.contains("API_KEY") {
        return true;
    }
    upper.split(|ch| ch == '_' || ch == '-').any(|part| {
        matches!(
            part,
            "TOKEN" | "SECRET" | "PASSWORD" | "PASSWD" | "CREDENTIAL" | "PASSPHRASE"
        )
    })
}

pub fn is_env_key(env: &str) -> bool {
    let mut chars = env.chars();
    match chars.next() {
        Some(first) if first.is_ascii_uppercase() || first == '_' => {
            chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        }
        _ => false,
    }
}

pub fn is_safe_dotenv_path(path: &str) -> bool {
    if path.is_empty() || path.contains('\0') {
        return false;
    }
    let parsed = std::path::Path::new(path);
    if parsed.is_absolute() {
        return false;
    }
    if parsed
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return false;
    }
    matches!(
        parsed.file_name().and_then(|name| name.to_str()),
        Some(name) if name == ".env" || name.starts_with(".env.")
    )
}

pub fn sanitize_field_ident(snake: &str) -> String {
    if is_reserved_ident(snake) {
        format!("flag_{snake}")
    } else {
        snake.to_string()
    }
}

fn is_reserved_ident(name: &str) -> bool {
    RESERVED_IDENTS.contains(&name)
}

const RESERVED_IDENTS: &[&str] = &[
    "abstract",
    "as",
    "assert",
    "async",
    "auto",
    "await",
    "become",
    "box",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "covariant",
    "crate",
    "debugger",
    "default",
    "deferred",
    "delete",
    "do",
    "dyn",
    "dynamic",
    "echo",
    "else",
    "enum",
    "export",
    "extends",
    "extension",
    "extern",
    "external",
    "factory",
    "false",
    "final",
    "finally",
    "fn",
    "for",
    "function",
    "get",
    "hide",
    "if",
    "impl",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "is",
    "late",
    "let",
    "library",
    "loop",
    "macro",
    "match",
    "mixin",
    "mod",
    "move",
    "mut",
    "new",
    "null",
    "on",
    "opaque",
    "operator",
    "override",
    "package",
    "panic",
    "part",
    "priv",
    "private",
    "protected",
    "pub",
    "public",
    "ref",
    "required",
    "rethrow",
    "return",
    "self",
    "set",
    "show",
    "static",
    "struct",
    "super",
    "switch",
    "sync",
    "this",
    "throw",
    "todo",
    "trait",
    "true",
    "try",
    "type",
    "typedef",
    "typeof",
    "union",
    "unsafe",
    "unsized",
    "use",
    "var",
    "virtual",
    "void",
    "where",
    "while",
    "with",
    "yield",
];

fn is_ident_start(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {
            chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_catalog, to_pascal, to_snake};

    #[test]
    fn snake_and_pascal_split_hyphens_and_case() {
        assert_eq!(to_snake("bind-addr"), "bind_addr");
        assert_eq!(to_snake("api_base"), "api_base");
        assert_eq!(to_pascal("bind-addr"), "BindAddr");
    }

    #[test]
    fn catalog_reads_identity_and_env_keys() {
        let catalog = parse_catalog(
            r#"
[identity]
service = "vxl-sidecar"
type_name = "SidecarEnv"

[flags.bind]
env = "VXL_SIDECAR_BIND"
type = "string"
default = "127.0.0.1:9090"
help = "HTTP listen address"
"#,
            None,
        )
        .expect("catalog");
        assert_eq!(catalog.service.as_deref(), Some("vxl-sidecar"));
        assert_eq!(catalog.type_name, "SidecarEnv");
        assert_eq!(catalog.flags.len(), 1);
        assert_eq!(catalog.flags[0].env, "VXL_SIDECAR_BIND");
        assert_eq!(catalog.flags[0].rust_const, "BIND");
        assert_eq!(catalog.flags[0].default.as_deref(), Some("127.0.0.1:9090"));
        assert!(!catalog.env_load.load);
        assert!(catalog.env_load.files.is_empty());
    }

    #[test]
    fn catalog_walks_nested_command_flags() {
        let catalog = parse_catalog(
            r#"
[commands.query.flags.dialect]
env = "CLARITAS_DIALECT"
default = "sql"
"#,
            Some("CliEnv"),
        )
        .expect("catalog");
        assert_eq!(catalog.flags.len(), 1);
        assert_eq!(catalog.flags[0].env, "CLARITAS_DIALECT");
        assert_eq!(catalog.flags[0].rust_const, "DIALECT");
    }

    #[test]
    fn catalog_reads_required_examples_and_dotenv_override() {
        let catalog = parse_catalog(
            r#"
[env]
load = true
files = [".env", ".env.local"]
override = false
order = ["flags", "env_shell", "env_file"]

[order-of-preference]
API_TOKEN = ["env_file", "env_shell", "flags"]

[flags.token]
env = "API_TOKEN"
required = true
examples = ["tok_live_123", "tok_test_abc"]
dotenv_override = true
"#,
            Some("CliEnv"),
        )
        .expect("catalog");
        assert!(catalog.env_load.load);
        assert_eq!(catalog.env_load.files, vec![".env", ".env.local"]);
        assert_eq!(catalog.flags[0].required, true);
        assert_eq!(
            catalog.flags[0].examples,
            vec!["tok_live_123", "tok_test_abc"]
        );
        assert_eq!(
            catalog.source_order_for(&catalog.flags[0]),
            vec!["env_file", "env_shell", "flags"]
        );
    }

    #[test]
    fn server_env_load_false_reads_no_dotenv_files() {
        let catalog = parse_catalog(
            r#"
[env]
load = false

[flags.bind]
env = "APP_BIND"
default = "127.0.0.1:8080"
"#,
            None,
        )
        .expect("catalog");
        assert!(!catalog.env_load.load);
        assert!(catalog.env_load.files.is_empty());
    }

    #[test]
    fn catalog_rejects_invalid_env_keys() {
        let err = parse_catalog(
            r#"
[flags.bind]
env = "not-an-env-key"
"#,
            None,
        )
        .unwrap_err();
        assert!(err.to_string().contains("not a valid ENV key"));
    }

    #[test]
    fn catalog_rejects_unsafe_dotenv_paths() {
        for files in [
            r#"files = ["../.env"]"#,
            r#"files = ["/etc/.env"]"#,
            r#"files = ["secrets.txt"]"#,
        ] {
            let toml = format!("[env]\nload = true\n{files}\n\n[flags.bind]\nenv = \"APP_BIND\"\n");
            let err = parse_catalog(&toml, None).unwrap_err();
            assert!(
                err.to_string().contains("refusing dotenv path"),
                "expected path refusal for {files}, got {err}"
            );
        }
    }

    #[test]
    fn catalog_sanitizes_reserved_field_names() {
        let catalog = parse_catalog(
            r#"
[flags.type]
env = "APP_TYPE"
default = "http"
"#,
            None,
        )
        .expect("catalog");
        assert_eq!(catalog.flags[0].snake, "flag_type");
        assert_eq!(catalog.flags[0].rust_const, "TYPE");
    }

    #[test]
    fn secret_examples_are_redacted() {
        let catalog = parse_catalog(
            r#"
[flags.token]
env = "API_TOKEN"
required = true
examples = ["tok_live_123"]
"#,
            None,
        )
        .expect("catalog");
        assert_eq!(super::example_values(&catalog.flags[0]), vec!["<redacted>"]);
        assert!(!super::is_secret_env("APP_BIND"));
        assert!(super::is_secret_env("CLIENT_SECRET"));
        assert!(!super::is_secret_env("SECRETARY_NAME"));
    }

    #[test]
    fn dotenv_path_sandbox_allows_only_relative_env_files() {
        assert!(super::is_safe_dotenv_path(".env"));
        assert!(super::is_safe_dotenv_path(".env.local"));
        assert!(super::is_safe_dotenv_path("config/.env.prod"));
        assert!(!super::is_safe_dotenv_path("../.env"));
        assert!(!super::is_safe_dotenv_path("/tmp/.env"));
        assert!(!super::is_safe_dotenv_path("notes.txt"));
        assert!(!super::is_safe_dotenv_path(".envrc"));
    }
}
