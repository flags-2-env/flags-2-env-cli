#![forbid(unsafe_code)]

use crate::args::CheckContractOpts;
use crate::config::Config;
use crate::contract::{check_os_map, failures_to_error};
use crate::error::CliError;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;

pub fn run(opts: &CheckContractOpts, _runtime: &Config) -> Result<(), CliError> {
    let text = fs::read_to_string(&opts.config).map_err(|err| {
        CliError::Config(format!("could not read {}: {err}", opts.config.display()))
    })?;
    let catalog = crate::catalog::parse_catalog(&text, None)?;
    let env = match &opts.json {
        Some(path) if path.as_os_str() == "-" => {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
            json_object_to_map(&buf)?
        }
        Some(path) => {
            let buf = fs::read_to_string(path).map_err(|err| {
                CliError::Config(format!("could not read {}: {err}", path.display()))
            })?;
            json_object_to_map(&buf)?
        }
        None => process_env_filtered(&catalog),
    };
    match check_os_map(&catalog, &env) {
        Ok(()) => {
            eprintln!(
                "check-contract ok: {} key(s) matched JSON Schema 2020-12",
                env.len()
            );
            Ok(())
        }
        Err(failures) => Err(failures_to_error(failures)),
    }
}

fn json_object_to_map(text: &str) -> Result<BTreeMap<String, String>, CliError> {
    let value: Value = serde_json::from_str(text)
        .map_err(|err| CliError::Config(format!("invalid env JSON: {err}")))?;
    let object = value
        .as_object()
        .ok_or_else(|| CliError::Config("env JSON must be an object of string values".into()))?;
    let mut out = BTreeMap::new();
    for (key, val) in object {
        let string = val
            .as_str()
            .map(str::to_string)
            .or_else(|| val.as_i64().map(|n| n.to_string()))
            .or_else(|| val.as_f64().map(|n| n.to_string()))
            .or_else(|| val.as_bool().map(|flag| flag.to_string()))
            .ok_or_else(|| {
                CliError::Config(format!("env JSON {key} must be a string, number, or bool"))
            })?;
        out.insert(key.clone(), string);
    }
    Ok(out)
}

fn process_env_filtered(catalog: &crate::catalog::Catalog) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for flag in &catalog.flags {
        if let Ok(value) = std::env::var(&flag.env) {
            if !value.trim().is_empty() {
                out.insert(flag.env.clone(), value);
            }
        }
    }
    out
}
