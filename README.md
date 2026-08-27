# flags-2-env-cli

Rust CLI for Canonical CLI flag to environment mapping used by ORESoftware runtimes. Uses flags-2-env via `.cli-flags.toml`. Talks to `flags-2-env-api-server.rs`.

Binaries: `f2e` and `flags2env-platform`.

```sh
f2e generate .cli-flags.toml --out generated --name CliEnv --lang rust,dart,typescript,gleam
```

`generate` writes compile-time env key constants and typed interfaces:

- `generated/rust/env.rs`
- `generated/dart/env.dart`
- `generated/typescript/env.ts`
- `generated/gleam/env.gleam`

Rust consumers include the generated keys instead of magic strings:

```rust
#[path = "../generated/rust/env.rs"]
mod env;

let bind = std::env::var(env::BIND).unwrap_or_else(|_| env::BIND_DEFAULT.to_string());
```
