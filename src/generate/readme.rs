#![forbid(unsafe_code)]

/// README written into `generated/` (frozen artifacts).
pub fn frozen() -> &'static str {
    FROZEN
}

/// README for a directory named `generated` that is *not* a freeze tree.
pub fn not_frozen() -> &'static str {
    NOT_FROZEN
}

const FROZEN: &str = r#"# `generated/` — frozen artifacts (read-only)

This tree is **generated**. Do not hand-edit anything here except this
README if you are documenting a local exception.

Typical producers:

- [`flags-2-env`](https://github.com/flags-2-env/flags-2-env-cli) (`f2e generate`)
- [`api-docs` / `ridl`](https://github.com/oresoftware/api-docs)
- interface adapters from `schema/tables.json` (`node src/generate.mjs`)

## Read-only on disk

After generate, artifact files are `chmod a-w` (0444). Directories stay
writable so the generator can add files. The generator unfreezes, writes,
then freezes again.

**Git does not store the Unix write bit** — only the executable bit
(100644 vs 100755). After `git clone` / `git checkout`, files come back
writable. Restore the policy with:

```sh
f2e generate          # or ridl generate / node src/generate.mjs
# or
chmod a-w generated/**/*.rs generated/**/*.ts generated/**/*.dart generated/**/*.json
# or
scripts/freeze-generated.sh
```

Do not `chmod u+w` and then commit a hand-edit. Change the source catalog
(`.cli-flags.toml`, route map, `schema/tables.json`) and regenerate.

## JSON Schema (the contract)

If `json-schema/` is present, those documents are JSON Schema 2020-12.
They are the interchange contract across Rust, TypeScript, and Dart.

- Compile-time types are generated *from* that catalog.
- Runtime `check_os_env` / `checkOsEnv` / `validate()` must pass on real
  payloads, not only on types that compile.
- Unit tests should feed **valid** and **invalid** instances (missing
  required keys, wrong types, extra properties).

```sh
f2e check-contract --config .cli-flags.toml --json env.fixture.json
```

## Gitignored trees

If this folder is listed in `.gitignore`, artifacts stay local. Keep this
README tracked with:

```
generated/*
!generated/README.md
```

(Do not ignore the directory node itself as `generated/` — that prevents
the `!README.md` exception from working.)

Regenerate after clone; CI should fail if checked-in artifacts drift.
"#;

const NOT_FROZEN: &str = r#"# `generated/` — not frozen

This directory is named `generated` but it is **not** a flags-2-env /
api-docs / interfaces freeze tree. Files here may stay writable. They are
not `chmod a-w` by policy.

Typical cases: compiler/tool caches, Flutter/Xcode `.generated/`, Gradle
`build/`, or scratch output that is not a source-of-truth adapter.

If you meant a frozen codegen tree, see the flags-2-env / api-docs README
template (`generated/` artifacts are 0444 after generate, JSON Schema is
the contract, runtime checkers must pass).
"#;
