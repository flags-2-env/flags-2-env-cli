// src/env/env.rs — service overlay. Edit defaults here; regenerate generated.rs from flags-2-env.

use super::generated;

/// Code-level defaults. Overlay values from flags-2-env (`.env` vs process env vs argv) win.
pub fn defaults() -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::from([])
}

/// Merge service defaults under the flags-2-env overlay.
/// Default rank: argv `flags` > `env_shell` > `env_file` (`.env`).
/// `dotenv_override` / `[env] override` lifts `.env` over the process environment.
/// Servers should set `[env] load = false` so a hostile CWD `.env` cannot inject values.
pub fn load() -> Result<std::collections::BTreeMap<String, String>, generated::MissingEnv> {
    let mut merged = defaults();
    merged.extend(generated::load_env_map_from_os()?);
    Ok(merged)
}

pub fn get<'a>(env: &'a std::collections::BTreeMap<String, String>, key: &str) -> Option<&'a str> {
    env.get(key)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_env_outranks_dotenv_by_default() {
        let mut shell = std::collections::BTreeMap::new();
        shell.insert("FLAGS_2_ENV_API_BASE".into(), "http://shell.test".into());
        let mut dotenv = std::collections::BTreeMap::new();
        dotenv.insert("FLAGS_2_ENV_API_BASE".into(), "http://file.test".into());
        let env = generated::load_env_map(&shell, &dotenv, &std::collections::BTreeMap::new())
            .expect("overlay");
        assert_eq!(
            env.get("FLAGS_2_ENV_API_BASE").map(String::as_str),
            Some("http://shell.test")
        );
    }

    #[test]
    fn dotenv_fills_when_process_env_is_empty() {
        let shell = std::collections::BTreeMap::new();
        let mut dotenv = std::collections::BTreeMap::new();
        dotenv.insert("FLAGS_2_ENV_API_BASE".into(), "http://file.test".into());
        let env = generated::load_env_map(&shell, &dotenv, &std::collections::BTreeMap::new())
            .expect("overlay");
        assert_eq!(
            env.get("FLAGS_2_ENV_API_BASE").map(String::as_str),
            Some("http://file.test")
        );
    }
}
