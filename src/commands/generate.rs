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
    let catalog = crate::catalog::parse_catalog(&text, Some(&config.type_name))?;
    for language in &config.languages {
        let dir = config.out_dir.join(crate::generate::dir_name(*language));
        fs::create_dir_all(&dir)?;
        let path = dir.join(crate::generate::file_name(*language));
        let source = crate::generate::render(*language, &catalog);
        write_if_changed(&path, &source)?;
        eprintln!("wrote {}", path.display());
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
