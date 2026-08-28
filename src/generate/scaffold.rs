#![forbid(unsafe_code)]

use crate::catalog::{example_values, Catalog};

pub fn readme() -> &'static str {
    README
}

pub fn rust_mod() -> &'static str {
    "#![allow(dead_code, unused_imports)]\n\nmod generated;\nmod env;\n\npub use generated::*;\npub use env::*;\n"
}

pub fn rust_env(catalog: &Catalog) -> String {
    let mut defaults = String::new();
    for flag in &catalog.flags {
        if let Some(default) = &flag.default {
            defaults.push_str(&format!(
                "        (\"{}\".to_string(), \"{}\".to_string()),\n",
                rust_escape(&flag.env),
                rust_escape(default)
            ));
        }
    }
    format!(
        r#"{header}
use super::generated;

/// Code-level defaults. Overlay values from flags-2-env (`.env` vs process env vs argv) win.
pub fn defaults() -> std::collections::BTreeMap<String, String> {{
    std::collections::BTreeMap::from([
{defaults}    ])
}}

/// Merge service defaults under the flags-2-env overlay.
/// Default rank: argv `flags` > `env_shell` > `env_file` (`.env`).
/// `dotenv_override` / `[env] override` lifts `.env` over the process environment.
/// Servers should set `[env] load = false` so a hostile CWD `.env` cannot inject values.
pub fn load() -> Result<std::collections::BTreeMap<String, String>, generated::MissingEnv> {{
    let mut merged = defaults();
    merged.extend(generated::load_env_map_from_os()?);
    Ok(merged)
}}

pub fn get<'a>(env: &'a std::collections::BTreeMap<String, String>, key: &str) -> Option<&'a str> {{
    env.get(key).map(String::as_str).map(str::trim).filter(|value| !value.is_empty())
}}
"#,
        header = rust_header(),
        defaults = defaults
    )
}

pub fn typescript_env(catalog: &Catalog) -> String {
    let mut defaults = String::new();
    for flag in &catalog.flags {
        let value = flag.default.as_deref().unwrap_or("");
        if !value.is_empty() {
            defaults.push_str(&format!(
                "    {}: \"{}\",\n",
                flag.env,
                ts_escape(value)
            ));
        }
    }
    let mut requires = String::new();
    for flag in &catalog.flags {
        if flag.required {
            requires.push_str(&format!(
                "  env[{key}] = generated.requireEnv({key}, \"{ty}\", {examples}, env[{key}]);\n",
                key = format!("\"{}\"", ts_escape(&flag.env)),
                ty = flag.flag_type,
                examples = ts_examples(flag),
            ));
        }
    }
    format!(
        r#"{header}
import * as generated from "./generated.ts";

const defaults: Record<string, string> = {{
{defaults}}};

export default {{
  get env(): Record<string, string> {{
    return load();
  }},
}};

export function load(
  shell: Record<string, string | undefined> = typeof process !== "undefined" ? process.env : {{}},
): Record<string, string> {{
  const env = Object.assign({{}}, defaults, generated.loadEnvMapFromOs(shell));
{requires}  return env;
}}

export {{ generated }};
"#,
        header = ts_header(),
        defaults = defaults,
        requires = requires
    )
}

pub fn dart_env(catalog: &Catalog) -> String {
    let mut defaults = String::new();
    for flag in &catalog.flags {
        if let Some(default) = &flag.default {
            defaults.push_str(&format!(
                "  '{}': '{}',\n",
                dart_escape(&flag.env),
                dart_escape(default)
            ));
        }
    }
    format!(
        r#"{header}
import 'generated.dart' as generated;

const Map<String, String> defaults = {{
{defaults}}};

/// Service defaults, then flags-2-env overlay (`.env` vs process env vs argv).
Map<String, String> load() {{
  return {{
    ...defaults,
    ...generated.loadEnvMapFromOs(),
  }};
}}
"#,
        header = dart_header(),
        defaults = defaults
    )
}

fn rust_header() -> &'static str {
    "// src/env/env.rs — service overlay. Edit defaults here; regenerate generated.rs from flags-2-env.\n"
}

fn ts_header() -> &'static str {
    "/* src/env/env.ts — service overlay. Edit defaults here; regenerate generated.ts from flags-2-env. */\n"
}

fn dart_header() -> &'static str {
    "// src/env/env.dart — service overlay. Edit defaults here; regenerate generated.dart from flags-2-env.\n"
}

fn rust_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn ts_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn dart_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn ts_examples(flag: &crate::catalog::FlagSpec) -> String {
    format!(
        "[{}]",
        example_values(flag)
            .iter()
            .map(|value| format!("\"{}\"", ts_escape(value)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

const README: &str = r#"# `src/env` — service environment overlay

Standard path for process configuration, with or without flags-2-env:

| Language | File |
| --- | --- |
| Rust | `src/env/env.rs` |
| TypeScript | `src/env/env.ts` |
| Dart | `src/env/env.dart` |

`main.rs` / `main.ts` / `main.dart` should import this module instead of
scattering `std::env::var` / `process.env` / `Platform.environment` reads.

## Layers (lowest → highest)

1. **Code defaults** in `env.rs` / `env.ts` / `env.dart` (safe local values).
2. **`.env` files** (`env_file`) — `./.env` by default, later files win.
3. **Process environment** (`env_shell`) — `PORT=7777 mycli` outranks `.env`.
4. **Argv flags** (`flags`) — only when flags-2-env parses a CLI.

flags-2-env is what ranks (2)–(4). Default rank is `flags > env_shell > env_file`.
That is why a live variable overrides `.env`, and why `--port` overrides both.

To lift `.env` over the process environment (still letting argv win):

```toml
[flags.token]
env = "API_TOKEN"
dotenv_override = true

# or for every key:
[env]
override = true
```

Per-key ranking:

```toml
[order-of-preference]
API_TOKEN = ["env_file", "env_shell", "flags"]
```

`FLAGS2ENV_DOTENV=0` can only **disable** file loading, never enable it.
Servers, MCP processes, and workers should set `[env] load = false` so a
hostile working-directory `.env` cannot inject values. Those processes take
deployment values from the real environment (and code defaults).

## Required values

If a required key is missing or empty after the overlay, throw and name:

- the exact env var (`DATABASE_URL`)
- the expected type (`string`, `int`, `bool`, …)
- examples of good values (`postgres://user:pass@127.0.0.1:5432/app`)

Do not depend on flags-2-env at runtime if the service has no CLI. Keep this
directory as the contract either way. When flags-2-env *is* used, `f2e generate
--src-env` writes `generated.rs` / `generated.ts` / `generated.dart` (do not
edit those) and scaffolds `env.*` once.

## Generate

```sh
f2e generate --src-env
f2e generate --src-env=src/env --lang rust,typescript,dart
```
"#;
