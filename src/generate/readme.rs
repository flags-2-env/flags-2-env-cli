#![forbid(unsafe_code)]

/// README written into `generated/` (frozen artifacts).
pub fn frozen() -> &'static str {
    FROZEN
}

/// README for a directory named `generated` that is *not* a freeze tree.
pub fn not_frozen() -> &'static str {
    NOT_FROZEN
}

const FROZEN: &str = r#"# `generated/` — frozen derivative artifacts (read-only)

This tree is **generated**. Do not hand-edit anything here except this
README when documenting a repository-specific exception.

Typical producers:

- [`flags-2-env`](https://github.com/flags-2-env/flags-2-env-cli) (`f2e generate`)
- [`api-docs` / `ridl`](https://github.com/oresoftware/api-docs)
- interface adapters from independently authored contract sources

## Authority classification

For flags-2-env output, `.cli-flags.toml` is the human-authored source for
CLI and process-environment configuration. Generated language bindings and
`json-schema/env.*.schema.json` files are derivative projections. The emitted
JSON Schema documents are runtime-validation witnesses; they are not an
independently human-authored domain or API authority.

For shared serialized domain, API, HTTP, RPC, event, persistence, or durable
storage contracts, TypeSpec and JSON Schema/OpenAPI must be independent,
human-authored peer authorities outside `generated/`. Neither may be generated
from or overwrite the other. Translations and round trips are comparison
evidence only. Any unexplained mismatch is `STOPPED_FOR_EVALUATION` and blocks
publication, merge, release, migration, and deployment.

Generated API documentation is also derivative. Its README must name the route
or contract inputs and the exact producer.

## Read-only on disk

After generation, artifact files are `chmod a-w` (normally 0444). A repository
may also freeze idle generated directories (normally 0555). The generator
unfreezes, writes, and freezes again.

**Git does not store the Unix write bit** — only the executable bit
(100644 vs 100755). After `git clone` or `git checkout`, files come back
writable. Restore the local policy with the pinned generator or the
repository's freeze command.

Do not `chmod u+w` and then commit a hand-edit. Change the documented
human-authored source and regenerate. Select only output languages with a
verified consumer or packaging pipeline.

## Validation and drift

Runtime checkers must exercise valid and invalid instances; compilation alone
is not sufficient.

```sh
f2e check-contract --config .cli-flags.toml --json env.fixture.json
```

CI must rerun the pinned generator and fail when
`git diff --exit-code -- generated/` reports drift.

## Gitignored trees

If this folder is listed in `.gitignore`, artifacts stay local. Keep this
README tracked with:

```
generated/*
!generated/README.md
```

Do not ignore the directory node itself as `generated/`, because that prevents
the README exception from working.
"#;

const NOT_FROZEN: &str = r#"# `generated/` — not a frozen source projection

This directory is named `generated` but is not a flags-2-env, api-docs, or
interface projection tree. Compiler caches, Flutter/Xcode `.generated/`,
Gradle `build/`, and scratch output may stay writable and should normally be
ignored rather than committed.

If this is actually checked-in derivative output, add the generated README,
provenance manifest, deterministic regeneration check, and local freeze
command. Classify its authority correctly: flags-2-env schemas are derived
validation witnesses, while shared domain/API contracts require independent
human-authored TypeSpec and JSON Schema/OpenAPI peer authorities.
"#;

#[cfg(test)]
mod tests {
    use super::{frozen, not_frozen};

    #[test]
    fn frozen_template_separates_configuration_witnesses_from_peer_contract_authorities() {
        let text = frozen();
        assert!(text.contains(".cli-flags.toml"));
        assert!(text.contains("runtime-validation witnesses"));
        assert!(text.contains("TypeSpec and JSON Schema/OpenAPI"));
        assert!(text.contains("STOPPED_FOR_EVALUATION"));
        assert!(!text.contains("JSON Schema (the contract)"));
    }

    #[test]
    fn non_frozen_template_does_not_reintroduce_single_authority_wording() {
        let text = not_frozen();
        assert!(text.contains("derived validation witnesses"));
        assert!(text.contains("independent"));
        assert!(!text.contains("JSON Schema is the contract"));
    }
}
