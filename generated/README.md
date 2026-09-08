<!-- generated-policy: frozen -->

# `generated/` — committed, and not hand-editable

Everything in this directory is machine-written and **committed to version
control**. Do not edit these files by hand — the only exception is this README,
if you are documenting a local exception. Change the source they come from and
re-run the generator.

Typical producers:

- [`flags-2-env`](https://github.com/flags-2-env/flags-2-env-cli) (`f2e generate`) —
  the Rust/TypeScript/Dart/Gleam adapters and `generated/json-schema/`
- [`api-docs` / `ridl`](https://github.com/oresoftware/api-docs) — route maps and clients
- interface adapters from `schema/tables.json` (`node src/generate.mjs`)

## Why the files are read-only on disk

After generation, artifact files are frozen with `chmod a-w` (0444). Directories
and this `README.md` stay writable so the generator can add and replace files;
the generator unfreezes, writes, then freezes again. Your editor will refuse the
write, which is the point — it turns "I edited the wrong file" into an error you
see immediately rather than a diff you notice in review.

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

Do not `chmod u+w` and then commit a hand-edit. Change the source catalog
(`.cli-flags.toml`, route map, `schema/tables.json`) and regenerate.

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

## JSON Schema (the contract)

The documents under `generated/json-schema/` are JSON Schema 2020-12 and are the
interchange contract across Rust, TypeScript, Dart and Gleam. The schema is an
independently derived description of the same contract as the generated types —
disagreement means one of them has drifted.

- Compile-time types are generated *from* the flag catalog.
- Runtime `check_os_env` / `checkOsEnv` / `validate()` must pass on real
  payloads, not only on types that compile.
- Unit tests should feed **valid** and **invalid** instances (missing required
  keys, wrong types, extra properties), and compare schema keys to
  `.cli-flags.toml` env names or route-map keys.

```sh
f2e check-contract --config .cli-flags.toml --json env.fixture.json
```

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
