<!-- generated-policy: frozen -->

# `generated/` — frozen derivative artifacts (read-only)

This tree is **generated**. Do not hand-edit anything here except this
README when documenting a repository-specific exception.

Everything in this directory is machine-written and **committed to version
control**. Typical producers:

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

## Why the files are read-only on disk

After generation, artifact files are frozen with `chmod a-w` (0444). Directories
and this `README.md` normally stay writable so the generator can add and replace
files; a repository may also freeze an idle generated directory (normally 0555).
The generator unfreezes, writes, then freezes again. Your editor will refuse a
direct artifact write, turning an accidental hand edit into an immediate error.

**Git does not store this.** Git tracks only the executable bit (100644 vs
100755), so a fresh `git clone` / `git checkout` comes back writable. The
read-only bit is a local ergonomic guard; it is *not* what enforces the policy.
Restore it with any of:

```sh
f2e generate          # or ridl generate / node src/generate.mjs
scripts/freeze-generated.sh
python3 scripts/check-generated-contract.py --freeze --require-readonly
chmod a-w generated/**/*.rs generated/**/*.ts generated/**/*.dart generated/**/*.gleam generated/**/*.json
```

Do not `chmod u+w` and then commit a hand-edit. Change the documented
human-authored source and regenerate. Select only output languages with a
verified consumer or packaging pipeline.

## Validation and drift

## What actually enforces the policy

CI, not the filesystem:

| Guard | Where | What it catches |
| --- | --- | --- |
| `check-generated-contract.py` | CI + pre-commit | a hand-edited or thawed file |
| regenerate-and-diff | CI | committed output that no longer matches its source |
| `post-checkout` / `post-merge` hooks | your clone | re-freezes after every checkout |

Enable the hooks once per clone:

```sh
git config core.hooksPath .githooks
```

## Regenerating

Edit the **primary source** — `.cli-flags.toml`, the route map, `*.schema.json`
— then run the generator. Generators thaw, write, and re-freeze on their own. If
you are committing a regeneration, the pre-commit guard needs to be told so:

```sh
REGEN=1 git commit -m "Regenerate adapters from the updated flag catalog"
```

CI should fail if checked-in artifacts drift from their source.

## Runtime validation witnesses

The documents under `generated/json-schema/` are JSON Schema 2020-12 and are the
runtime-validation projection of the human-authored `.cli-flags.toml` contract
across Rust, TypeScript, Dart and Gleam. The schema and generated types must
agree with that source; disagreement means one of the derivative artifacts has
drifted.

- Compile-time types are generated *from* the flag catalog.
- Runtime `check_os_env` / `checkOsEnv` / `validate()` must pass on real
  payloads, not only on types that compile.
- Unit tests should feed **valid** and **invalid** instances (missing required
  keys, wrong types, extra properties), and compare schema keys to
  `.cli-flags.toml` env names or route-map keys.

```sh
f2e check-contract --config .cli-flags.toml --json env.fixture.json
```

CI must rerun the pinned generator and fail when
`git diff --exit-code -- generated/` reports drift.

## Gitignored trees

If a `generated/` folder is listed in `.gitignore`, its artifacts stay local and
the tree's policy is `ignored`, not `frozen`. Still commit the README so the
policy stays visible — `git add -f generated/README.md`, or a `.gitignore`
exception:

```
generated/*
!generated/README.md
```

(Do not ignore the directory node itself as `generated/` — that prevents the
`!README.md` exception from working.)
