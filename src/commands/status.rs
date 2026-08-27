#![forbid(unsafe_code)]

use crate::config::Config;
use crate::error::CliError;

pub fn run(config: &Config) -> Result<(), CliError> {
    println!("{}", status_output(config));
    Ok(())
}

fn status_output(config: &Config) -> String {
    let body = serde_json::json!({
        "service": "flags-2-env",
        "api_base": config.api_base,
    });
    match config.json {
        true => body.to_string(),
        false => format!("flags-2-env @ {}", config.api_base),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn json_and_plain_outputs_are_pure_transformations() {
        let json = Config {
            api_base: "http://127.0.0.1:8080".into(),
            json: true,
        };
        let plain = Config {
            api_base: "http://127.0.0.1:8080".into(),
            json: false,
        };
        assert_eq!(
            status_output(&json),
            r#"{"api_base":"http://127.0.0.1:8080","service":"flags-2-env"}"#
        );
        assert_eq!(status_output(&plain), "flags-2-env @ http://127.0.0.1:8080");
    }
}
