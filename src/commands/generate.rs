#![forbid(unsafe_code)]

use crate::config::Config;
use crate::error::CliError;
use std::fs;
use std::path::Path;

pub fn run(config: &crate::args::GenerateOpts, _runtime: &Config) -> Result<(), CliError> {
    let text = fs::read_to_string(&config.config).map_err(|err| {
        CliError::Config(format!("could not read {}: {err}", config.config.display()))
    })?;
    let catalog = crate::catalog::parse_catalog(
        &text,
        if config.type_name != "CliEnv" {
            Some(config.type_name.as_str())
        } else {
            None
        },
    )?;
    for language in &config.languages {
        let dir = config.out_dir.join(crate::generate::dir_name(*language));
        fs::create_dir_all(&dir)?;
        let types_path = dir.join(crate::generate::file_name(*language));
        let types_source = crate::generate::render(*language, &catalog);
        crate::generate::write_frozen(&types_path, &types_source)?;
        eprintln!("wrote {}", types_path.display());
        let runtime_path = dir.join(crate::generate::runtime_file_name(*language));
        let runtime_source = crate::generate::render_runtime(*language, &catalog);
        crate::generate::write_frozen(&runtime_path, &runtime_source)?;
        eprintln!("wrote {}", runtime_path.display());
        if *language == crate::args::Language::Dart {
            for (name, source) in crate::generate::dart_platform_files() {
                let path = dir.join(name);
                crate::generate::write_frozen(&path, source)?;
                eprintln!("wrote {}", path.display());
            }
        }
    }
    let schema_dir = config.out_dir.join("json-schema");
    crate::generate::write_frozen(
        &schema_dir.join("env.os.schema.json"),
        &crate::generate::render_os_schema(&catalog),
    )?;
    crate::generate::write_frozen(
        &schema_dir.join("env.values.schema.json"),
        &crate::generate::render_values_schema(&catalog),
    )?;
    crate::generate::write_frozen(
        &config.out_dir.join("README.md"),
        crate::generate::generated_readme(),
    )?;
    eprintln!("wrote {}", schema_dir.join("env.os.schema.json").display());
    if let Some(src_env) = &config.src_env {
        write_src_env(src_env, &config.languages, &catalog)?;
    }
    Ok(())
}

fn write_src_env(
    src_env: &Path,
    languages: &[crate::args::Language],
    catalog: &crate::catalog::Catalog,
) -> Result<(), CliError> {
    fs::create_dir_all(src_env)?;
        crate::generate::write_frozen(
            &src_env.join("readme.md"),
            crate::generate::scaffold_readme(),
        )?;
    eprintln!("wrote {}", src_env.join("readme.md").display());
    if languages.contains(&crate::args::Language::Rust) {
        write_if_absent(src_env.join("mod.rs"), crate::generate::scaffold_mod_rs())?;
    }
    for language in languages {
        let generated = combined_generated(*language, catalog);
        let generated_path = src_env.join(crate::generate::generated_flat_name(*language));
        crate::generate::write_frozen(&generated_path, &generated)?;
        eprintln!("wrote {}", generated_path.display());
        if let Some(wrapper) = crate::generate::scaffold_env(*language, catalog) {
            write_if_absent(src_env.join(crate::generate::file_name(*language)), wrapper)?;
        }
    }
    if languages.contains(&crate::args::Language::Dart) {
        for (name, source) in crate::generate::dart_platform_files() {
            crate::generate::write_frozen(&src_env.join(name), source)?;
            eprintln!("wrote {}", src_env.join(name).display());
        }
    }
    Ok(())
}

fn combined_generated(
    language: crate::args::Language,
    catalog: &crate::catalog::Catalog,
) -> String {
    let types = crate::generate::render(language, catalog);
    let runtime = crate::generate::render_runtime(language, catalog);
    match language {
        crate::args::Language::Rust => {
            format!("{types}\n{}\n", strip_inner_attributes(&runtime))
        }
        crate::args::Language::Dart => hoist_dart_imports(&types, &runtime),
        _ => format!("{types}\n{runtime}\n"),
    }
}

fn hoist_dart_imports(types: &str, runtime: &str) -> String {
    let mut imports = Vec::new();
    let mut body = String::new();
    for source in [types, runtime] {
        for line in source.lines() {
            if line.trim_start().starts_with("import ") {
                if !imports.iter().any(|existing| existing == line) {
                    imports.push(line.to_string());
                }
            } else {
                body.push_str(line);
                body.push('\n');
            }
        }
    }
    if imports.is_empty() {
        return body;
    }
    format!("{}\n\n{body}", imports.join("\n"))
}

fn strip_inner_attributes(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("#!["))
        .collect::<Vec<_>>()
        .join("\n")
}

fn write_if_absent(path: impl AsRef<Path>, source: impl AsRef<str>) -> Result<(), CliError> {
    let path = path.as_ref();
    if path.exists() {
        eprintln!("kept {}", path.display());
        return Ok(());
    }
    fs::write(path, source.as_ref())?;
    eprintln!("wrote {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_src_env;
    use crate::args::Language;
    use crate::catalog::parse_catalog;
    use std::fs;

    #[test]
    fn src_env_writes_readme_and_language_wrappers() {
        let dir = std::env::temp_dir().join(format!("f2e-src-env-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let catalog = parse_catalog(
            r#"
[identity]
service = "demo"

[env]
load = false

[flags.bind]
env = "DEMO_BIND"
default = "127.0.0.1:8080"

[flags.token]
env = "DEMO_TOKEN"
required = true
examples = ["tok_live_123"]
"#,
            Some("DemoEnv"),
        )
        .unwrap();
        write_src_env(
            &dir,
            &[Language::Rust, Language::TypeScript, Language::Dart],
            &catalog,
        )
        .unwrap();
        let readme = fs::read_to_string(dir.join("readme.md")).unwrap();
        assert!(readme.contains("flags > env_shell > env_file"));
        let rust = fs::read_to_string(dir.join("generated.rs")).unwrap();
        assert!(rust.contains("DEMO_BIND"));
        assert!(rust.contains("missing required environment variable"));
        assert!(rust.contains("<redacted>"));
        assert!(!rust.contains("tok_live_123"));
        assert!(rust.contains("try_load_from_os"));
        assert!(rust.contains("load_env_map"));
        assert!(dir.join("env.rs").exists());
        assert!(dir.join("env.ts").exists());
        assert!(dir.join("env_io.dart").exists());
        assert!(dir.join("env_io_stub.dart").exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
