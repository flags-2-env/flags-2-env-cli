#![forbid(unsafe_code)]

use crate::config::Config;
use crate::error::CliError;

pub fn run(config: &Config) -> Result<(), CliError> {
    println!("{}", health_output(config));
    Ok(())
}

fn health_output(config: &Config) -> String {
    let body = serde_json::json!({
        "ok": true,
        "api_base": config.api_base,
    });
    match config.json {
        true => body.to_string(),
        false => format!("ok {}", config.api_base),
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
            health_output(&json),
            r#"{"api_base":"http://127.0.0.1:8080","ok":true}"#
        );
        assert_eq!(health_output(&plain), "ok http://127.0.0.1:8080");
    }
}
