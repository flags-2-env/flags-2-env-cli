#![forbid(unsafe_code)]

use crate::error::CliError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Health,
    Status,
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
    items.try_fold(
        Invocation {
            command,
            api_base: None,
            json: false,
        },
        apply_flag,
    )
}

fn take_command(
    items: &mut std::iter::Peekable<impl Iterator<Item = String>>,
) -> Result<Command, CliError> {
    match items.peek().map(String::as_str) {
        Some("health") => consume(items, Command::Health),
        Some("status") => consume(items, Command::Status),
        Some("-h" | "--help" | "help") => consume(items, Command::Help),
        Some(other) if !other.starts_with('-') => {
            Err(CliError::Usage(format!("unknown command {other}")))
        }
        Some(_) | None => Ok(Command::Help),
    }
}

fn consume(
    items: &mut std::iter::Peekable<impl Iterator<Item = String>>,
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

pub fn help_text() -> &'static str {
    "flags2env-platform — flags-2-env CLI\n\nCommands:\n  health\n  status\n"
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
}
