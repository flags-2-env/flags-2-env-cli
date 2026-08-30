#![forbid(unsafe_code)]

use crate::error::CliError;
use std::iter::Peekable;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Rust,
    Dart,
    TypeScript,
    Gleam,
}

impl Language {
    pub const ALL: [Self; 4] = [Self::Rust, Self::Dart, Self::TypeScript, Self::Gleam];

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
        Self::ALL.to_vec()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenerateOpts {
    pub config: PathBuf,
    pub out_dir: PathBuf,
    pub type_name: String,
    pub languages: Vec<Language>,
    pub src_env: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckContractOpts {
    pub config: PathBuf,
    pub json: Option<PathBuf>,
}

impl GenerateOpts {
    fn defaults() -> Self {
        Self {
            config: PathBuf::from(".cli-flags.toml"),
            out_dir: PathBuf::from("generated"),
            type_name: "CliEnv".into(),
            languages: Language::all(),
            src_env: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Health,
    Status,
    Generate(GenerateOpts),
    CheckContract(CheckContractOpts),
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
    let mut items = args.into_iter().peekable();
    let command = take_command(&mut items)?;
    match command {
        Command::Generate(_) => Ok(Invocation {
            command: Command::Generate(parse_generate(&mut items)?),
            api_base: None,
            json: false,
        }),
        Command::CheckContract(_) => Ok(Invocation {
            command: Command::CheckContract(parse_check_contract(&mut items)?),
            api_base: None,
            json: false,
        }),
        command @ (Command::Help | Command::Health | Command::Status) => items.try_fold(
            Invocation {
                command,
                api_base: None,
                json: false,
            },
            apply_flag,
        ),
    }
}

fn take_command(items: &mut Peekable<impl Iterator<Item = String>>) -> Result<Command, CliError> {
    match items.peek().map(String::as_str) {
        Some("health") => consume(items, Command::Health),
        Some("status") => consume(items, Command::Status),
        Some("generate") => consume(items, Command::Generate(GenerateOpts::defaults())),
        Some("check-contract") => consume(
            items,
            Command::CheckContract(CheckContractOpts {
                config: PathBuf::from(".cli-flags.toml"),
                json: None,
            }),
        ),
        Some("-h" | "--help" | "help") => consume(items, Command::Help),
        Some(other) if !other.starts_with('-') => {
            Err(CliError::Usage(format!("unknown command {other}")))
        }
        Some(_) | None => Ok(Command::Help),
    }
}

fn consume(
    items: &mut Peekable<impl Iterator<Item = String>>,
    command: Command,
) -> Result<Command, CliError> {
    items.next();
    Ok(command)
}

fn apply_flag(invocation: Invocation, arg: String) -> Result<Invocation, CliError> {
    match arg.as_str() {
        "--json" => Ok(Invocation {
            json: true,
            ..invocation
        }),
        "--help" | "-h" => Ok(Invocation {
            command: Command::Help,
            ..invocation
        }),
        flag if flag.starts_with("--api-base=") => Ok(Invocation {
            api_base: Some(flag.trim_start_matches("--api-base=").to_string()),
            ..invocation
        }),
        other => Err(CliError::Usage(format!("unknown flag {other}"))),
    }
}

fn parse_generate(
    items: &mut Peekable<impl Iterator<Item = String>>,
) -> Result<GenerateOpts, CliError> {
    let opts = parse_generate_from(GenerateOpts::defaults(), items)?;
    if opts.type_name.trim().is_empty() {
        return Err(CliError::Usage("--name must not be empty".into()));
    }
    Ok(opts)
}

fn parse_generate_from(
    opts: GenerateOpts,
    items: &mut Peekable<impl Iterator<Item = String>>,
) -> Result<GenerateOpts, CliError> {
    let Some(arg) = items.next() else {
        return Ok(opts);
    };
    let opts = apply_generate_arg(opts, arg, items)?;
    parse_generate_from(opts, items)
}

fn apply_generate_arg(
    opts: GenerateOpts,
    arg: String,
    items: &mut Peekable<impl Iterator<Item = String>>,
) -> Result<GenerateOpts, CliError> {
    match arg.as_str() {
        "--help" | "-h" => Err(CliError::Usage(help_text().into())),
        "--config" => Ok(GenerateOpts {
            config: required_value("--config", items.next())?.into(),
            ..opts
        }),
        flag if flag.starts_with("--config=") => Ok(GenerateOpts {
            config: flag.trim_start_matches("--config=").into(),
            ..opts
        }),
        "--out" => Ok(GenerateOpts {
            out_dir: required_value("--out", items.next())?.into(),
            ..opts
        }),
        flag if flag.starts_with("--out=") => Ok(GenerateOpts {
            out_dir: flag.trim_start_matches("--out=").into(),
            ..opts
        }),
        "--name" => Ok(GenerateOpts {
            type_name: required_value("--name", items.next())?.to_string(),
            ..opts
        }),
        flag if flag.starts_with("--name=") => Ok(GenerateOpts {
            type_name: flag.trim_start_matches("--name=").to_string(),
            ..opts
        }),
        "--lang" => Ok(GenerateOpts {
            languages: parse_languages(&required_value("--lang", items.next())?)?,
            ..opts
        }),
        flag if flag.starts_with("--lang=") => Ok(GenerateOpts {
            languages: parse_languages(flag.trim_start_matches("--lang="))?,
            ..opts
        }),
        "--src-env" => Ok(GenerateOpts {
            src_env: Some(peek_optional_path(items).into()),
            ..opts
        }),
        flag if flag.starts_with("--src-env=") => Ok(GenerateOpts {
            src_env: Some(flag.trim_start_matches("--src-env=").into()),
            ..opts
        }),
        other if !other.starts_with('-') => Ok(GenerateOpts {
            config: PathBuf::from(other),
            ..opts
        }),
        other => Err(CliError::Usage(format!("unknown generate flag {other}"))),
    }
}

fn peek_optional_path(items: &mut Peekable<impl Iterator<Item = String>>) -> String {
    match items.peek() {
        Some(next) if !next.is_empty() && !next.starts_with('-') => items.next().unwrap(),
        _ => "src/env".into(),
    }
}

fn required_value(flag: &str, value: Option<String>) -> Result<String, CliError> {
    value
        .filter(|text| !text.is_empty() && !text.starts_with('-'))
        .ok_or_else(|| CliError::Usage(format!("{flag} requires a value")))
}

fn parse_languages(value: &str) -> Result<Vec<Language>, CliError> {
    let languages = value
        .split(',')
        .map(Language::parse)
        .collect::<Result<Vec<_>, _>>()?;
    let mut unique = Vec::new();
    for language in languages {
        if !unique.contains(&language) {
            unique.push(language);
        }
    }
    if unique.is_empty() {
        return Err(CliError::Usage(
            "--lang requires rust, dart, typescript, and/or gleam".into(),
        ));
    }
    Ok(unique)
}

fn parse_check_contract(
    items: &mut Peekable<impl Iterator<Item = String>>,
) -> Result<CheckContractOpts, CliError> {
    let mut opts = CheckContractOpts {
        config: PathBuf::from(".cli-flags.toml"),
        json: None,
    };
    while let Some(arg) = items.next() {
        match arg.as_str() {
            "--config" => {
                opts.config = required_value("--config", items.next())?.into();
            }
            flag if flag.starts_with("--config=") => {
                opts.config = flag.trim_start_matches("--config=").into();
            }
            "--json" => {
                opts.json = Some(required_value("--json", items.next())?.into());
            }
            flag if flag.starts_with("--json=") => {
                opts.json = Some(flag.trim_start_matches("--json=").into());
            }
            other if !other.starts_with('-') => {
                opts.config = PathBuf::from(other);
            }
            other => {
                return Err(CliError::Usage(format!(
                    "unknown check-contract flag {other}"
                )))
            }
        }
    }
    Ok(opts)
}

pub fn help_text() -> &'static str {
    "f2e / flags2env-platform — flags-2-env CLI\n\n\
Commands:\n  \
  health\n  \
  status\n  \
  generate [config] [--out DIR] [--name TypeName] [--lang rust,dart,typescript,gleam]\n  \
  check-contract [config] [--json FILE|-]\n\n\
generate writes compile-time env key types plus JSON Schema and a runtime checker.\n  \
  --src-env [DIR]  also scaffold src/env/env.{rs,ts,dart} with .env vs process-env overlay\n\
check-contract validates a JSON object of env vars against the 2020-12 schema.\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_line(line: &str) -> Result<Invocation, CliError> {
        parse(line.split_whitespace().map(str::to_string))
    }

    #[test]
    fn defaults_to_help_without_a_command() {
        assert_eq!(
            parse_line("").unwrap(),
            Invocation {
                command: Command::Help,
                api_base: None,
                json: false,
            }
        );
    }

    #[test]
    fn folds_flags_without_mutating_an_accumulator() {
        assert_eq!(
            parse_line("status --json --api-base=http://127.0.0.1:9").unwrap(),
            Invocation {
                command: Command::Status,
                api_base: Some("http://127.0.0.1:9".into()),
                json: true,
            }
        );
    }

    #[test]
    fn rejects_unknown_commands_and_flags() {
        assert!(matches!(parse_line("migrate"), Err(CliError::Usage(_))));
        assert!(matches!(
            parse_line("health --quiet"),
            Err(CliError::Usage(_))
        ));
    }

    #[test]
    fn trailing_help_overrides_the_selected_command() {
        assert_eq!(parse_line("health --help").unwrap().command, Command::Help);
    }

    #[test]
    fn generate_defaults_cover_all_languages() {
        let invocation = parse(["generate".into()]).expect("parse");
        let Command::Generate(opts) = invocation.command else {
            panic!("expected generate");
        };
        assert_eq!(opts.config.as_os_str(), ".cli-flags.toml");
        assert_eq!(opts.out_dir.as_os_str(), "generated");
        assert_eq!(opts.type_name, "CliEnv");
        assert_eq!(opts.languages, Language::all());
    }

    #[test]
    fn generate_accepts_positional_config_and_lang_subset() {
        let invocation = parse([
            "generate".into(),
            "flags.toml".into(),
            "--name".into(),
            "SidecarEnv".into(),
            "--lang=rust,gleam".into(),
        ])
        .expect("parse");
        let Command::Generate(opts) = invocation.command else {
            panic!("expected generate");
        };
        assert_eq!(opts.config.as_os_str(), "flags.toml");
        assert_eq!(opts.type_name, "SidecarEnv");
        assert_eq!(opts.languages, vec![Language::Rust, Language::Gleam]);
        assert_eq!(opts.src_env, None);
    }

    #[test]
    fn generate_src_env_defaults_to_src_env() {
        let invocation = parse(["generate".into(), "--src-env".into()]).expect("parse");
        let Command::Generate(opts) = invocation.command else {
            panic!("expected generate");
        };
        assert_eq!(
            opts.src_env.as_deref().map(|path| path.as_os_str()),
            Some(std::ffi::OsStr::new("src/env"))
        );
    }
}
