# Finite CLI/env/dotenv precedence model and Atto source seed

`precedence_model.cjs` enumerates all eight presence masks over the documented three sources (CLI flag, shell env, dotenv) in both enabled/disabled dotenv modes, under both normal and dotenv-overrides-shell orderings: 16 input configurations, 32 decisions.

Properties: CLI flag always wins, dotenv opt-out is absolute, only an actually present and enabled source can win, no source is invented, and both shell-over-dotenv and dotenv-over-shell paths are reachable. A deliberate disabled-dotenv mutation must be rejected.

The model follows the **default source precedence** in `src/catalog.rs` and does **not** prove every custom `order-of-preference` TOML override, CLI parser, nested dotenv paths, symlink safety, typed coercions, or environment side effects. Replay against actual Rust `Catalog` and generated Rust/Dart/TypeScript/Gleam interfaces before promotion.

`flags_env_policy.atto` is a pure AttoLang source-level selector, not a generator integration, environment read, or guest execution. Before calling AttoLang a fully supported output backend, add `--lang attolang` behind reviewed grammar/manifest contracts and compare generated `.atto` names/values to all other native projections with exact-head compiler-backed E2E receipts.

Run: `node formal/atto-env/precedence_model.cjs`.
