#![forbid(unsafe_code)]

use crate::error::CliError;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Rust,
    Dart,
    TypeScript,
    Gleam,
}

impl Language {
    pub fn parse(value: &str) -> Result<Self, CliError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "rust" | "rs" => Ok(Self::Rust),
            "dart" => Ok(Self::Dart),
            "typescript" | "ts" => Ok(Self::TypeScript),
            "gleam" | "gleamlang" => Ok(Self::Gleam),
            other => Err(CliError::Usage(format!(
                "unknown generate language {other} (rust, dart, typescript, gleam)"
            ))),
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Rust, Self::Dart, Self::TypeScript, Self::Gleam]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenerateOpts {
    pub config: PathBuf,
    pub out_dir: PathBuf,
    pub type_name: String,
    pub languages: Vec<Language>,
}

impl GenerateOpts {
    fn defaults() -> Self {
        Self {
            config: PathBuf::from(".cli-flags.toml"),
            out_dir: PathBuf::from("generated"),
            type_name: "CliEnv".into(),
            languages: Language::all(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Health,
    Status,
    Generate(GenerateOpts),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    pub command: Command,
    pub api_base: Option<String>,
    pub json: bool,
}

pub fn parse<I>(args: I) -> Result<Invocation, CliError>
where
    I: IntoIterator<Item = String>,
{
    let mut command = Command::Help;
    let mut api_base = None;
    let mut json = false;
    let mut items = args.into_iter().peekable();
    if let Some(first) = items.peek() {
        match first.as_str() {
            "health" => {
                command = Command::Health;
                items.next();
            }
            "status" => {
                command = Command::Status;
                items.next();
            }
            "generate" => {
                items.next();
                command = Command::Generate(parse_generate(items)?);
                return Ok(Invocation {
                    command,
                    api_base: None,
                    json: false,
                });
            }
            "-h" | "--help" | "help" => {
                command = Command::Help;
                items.next();
            }
            other if !other.starts_with('-') => {
                return Err(CliError::Usage(format!("unknown command {other}")));
            }
            _ => {}
        }
    }
    for arg in items {
        match arg.as_str() {
            "--json" => json = true,
            "--help" | "-h" => command = Command::Help,
            flag if flag.starts_with("--api-base=") => {
                api_base = Some(flag.trim_start_matches("--api-base=").to_string());
            }
            other => return Err(CliError::Usage(format!("unknown flag {other}"))),
        }
    }
    Ok(Invocation {
        command,
        api_base,
        json,
    })
}

fn parse_generate<I>(items: I) -> Result<GenerateOpts, CliError>
where
    I: IntoIterator<Item = String>,
{
    let mut opts = GenerateOpts::defaults();
    let mut items = items.into_iter();
    while let Some(arg) = items.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                return Err(CliError::Usage(help_text().into()));
            }
            "--config" => {
                opts.config = required_value("--config", items.next())?.into();
            }
            flag if flag.starts_with("--config=") => {
                opts.config = flag.trim_start_matches("--config=").into();
            }
            "--out" => {
                opts.out_dir = required_value("--out", items.next())?.into();
            }
            flag if flag.starts_with("--out=") => {
                opts.out_dir = flag.trim_start_matches("--out=").into();
            }
            "--name" => {
                opts.type_name = required_value("--name", items.next())?.to_string();
            }
            flag if flag.starts_with("--name=") => {
                opts.type_name = flag.trim_start_matches("--name=").to_string();
            }
            "--lang" => {
                opts.languages = parse_languages(&required_value("--lang", items.next())?)?;
            }
            flag if flag.starts_with("--lang=") => {
                opts.languages = parse_languages(flag.trim_start_matches("--lang="))?;
            }
            other if !other.starts_with('-') => {
                opts.config = PathBuf::from(other);
            }
            other => return Err(CliError::Usage(format!("unknown generate flag {other}"))),
        }
    }
    if opts.type_name.trim().is_empty() {
        return Err(CliError::Usage("--name must not be empty".into()));
    }
    Ok(opts)
}

fn required_value(flag: &str, value: Option<String>) -> Result<String, CliError> {
    value
        .filter(|text| !text.is_empty() && !text.starts_with('-'))
        .ok_or_else(|| CliError::Usage(format!("{flag} requires a value")))
}

fn parse_languages(value: &str) -> Result<Vec<Language>, CliError> {
    let mut languages = Vec::new();
    for part in value.split(',') {
        let language = Language::parse(part)?;
        if !languages.contains(&language) {
            languages.push(language);
        }
    }
    if languages.is_empty() {
        return Err(CliError::Usage(
            "--lang requires rust, dart, typescript, and/or gleam".into(),
        ));
    }
    Ok(languages)
}

pub fn help_text() -> &'static str {
    "f2e / flags2env-platform — flags-2-env CLI\n\n\
Commands:\n  \
  health\n  \
  status\n  \
  generate [config] [--out DIR] [--name TypeName] [--lang rust,dart,typescript,gleam]\n\n\
generate writes compile-time env key types/interfaces/vars from .cli-flags.toml.\n"
}

#[cfg(test)]
mod tests {
    use super::{parse, Command, Language};

    #[test]
    fn generate_defaults_cover_all_languages() {
        let invocation = parse(["generate".into()]).expect("parse");
        match invocation.command {
            Command::Generate(opts) => {
                assert_eq!(opts.config.as_os_str(), ".cli-flags.toml");
                assert_eq!(opts.out_dir.as_os_str(), "generated");
                assert_eq!(opts.type_name, "CliEnv");
                assert_eq!(opts.languages, Language::all());
            }
            other => panic!("expected generate, got {other:?}"),
        }
    }

    #[test]
    fn generate_accepts_positional_config_and_lang_subset() {
        let invocation = parse(
            [
                "generate".into(),
                "flags.toml".into(),
                "--name".into(),
                "SidecarEnv".into(),
                "--lang=rust,gleam".into(),
            ]
            .into_iter(),
        )
        .expect("parse");
        match invocation.command {
            Command::Generate(opts) => {
                assert_eq!(opts.config.as_os_str(), "flags.toml");
                assert_eq!(opts.type_name, "SidecarEnv");
                assert_eq!(opts.languages, vec![Language::Rust, Language::Gleam]);
            }
            other => panic!("expected generate, got {other:?}"),
        }
    }
}
