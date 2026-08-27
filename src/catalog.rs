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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    pub service: Option<String>,
    pub type_name: String,
    pub flags: Vec<FlagSpec>,
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
    if let Some(flag_tables) = table.get("flags").and_then(|value| value.as_table()) {
        for (name, value) in flag_tables {
            let Some(flag_table) = value.as_table() else {
                continue;
            };
            flags.push(flag_from_table(name, flag_table)?);
        }
    }
    if flags.is_empty() {
        return Err(CliError::Config(
            "no [flags.*] entries with env keys found".into(),
        ));
    }
    Ok(Catalog {
        service,
        type_name,
        flags,
    })
}

fn flag_from_table(name: &str, table: &toml::Table) -> Result<FlagSpec, CliError> {
    let env = table
        .get("env")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| CliError::Config(format!("flag {name} is missing env")))?
        .to_string();
    let snake = to_snake(name);
    let rust_const = snake.to_ascii_uppercase();
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
    Ok(FlagSpec {
        name: name.to_string(),
        env,
        rust_const,
        snake,
        flag_type,
        default,
        help,
    })
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
    }
}
