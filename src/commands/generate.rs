#![forbid(unsafe_code)]

use crate::config::Config;
use crate::error::CliError;
use std::fs;
use std::path::Path;

pub fn run(config: &crate::args::GenerateOpts, _runtime: &Config) -> Result<(), CliError> {
    let text = fs::read_to_string(&config.config).map_err(|err| {
        CliError::Config(format!(
            "could not read {}: {err}",
            config.config.display()
        ))
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
        write_if_changed(&types_path, &types_source)?;
        eprintln!("wrote {}", types_path.display());
        let runtime_path = dir.join(crate::generate::runtime_file_name(*language));
        let runtime_source = crate::generate::render_runtime(*language, &catalog);
        write_if_changed(&runtime_path, &runtime_source)?;
        eprintln!("wrote {}", runtime_path.display());
    }
    Ok(())
}

fn write_if_changed(path: &Path, source: &str) -> Result<(), CliError> {
    match fs::read_to_string(path) {
        Ok(existing) if existing == source => Ok(()),
        _ => {
            fs::write(path, source)?;
            Ok(())
        }
    }
}
