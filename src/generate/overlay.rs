#![forbid(unsafe_code)]

use crate::catalog::{example_values, Catalog, FlagSpec};

pub fn render_rust(catalog: &Catalog) -> String {
    let mut out = String::new();
    out.push_str("\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct MissingEnv {\n");
    out.push_str("    pub name: &'static str,\n    pub expected_type: &'static str,\n    pub examples: &'static [&'static str],\n}\n\n");
    out.push_str("impl std::fmt::Display for MissingEnv {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
    out.push_str("        write!(f, \"missing required environment variable {}\\n  expected type: {}\\n  examples: {}\", self.name, self.expected_type, self.examples.join(\", \"))\n    }\n}\n\n");
    out.push_str("impl std::error::Error for MissingEnv {}\n");
    out.push_str(RUST_HELPERS);
    out.push_str("\n/// Resolve env-key -> value. Empty values fall through to the next source.\n");
    out.push_str("pub fn load_env_map(\n    shell: &std::collections::BTreeMap<String, String>,\n    dotenv: &std::collections::BTreeMap<String, String>,\n    flags: &std::collections::BTreeMap<String, String>,\n) -> Result<std::collections::BTreeMap<String, String>, MissingEnv> {\n");
    out.push_str("    let mut out = std::collections::BTreeMap::new();\n");
    for flag in &catalog.flags {
        let order = rust_order_array(&catalog.source_order_for(flag));
        let default = match &flag.default {
            Some(value) => format!("Some(\"{}\")", rust_escape(value)),
            None => "None".into(),
        };
        out.push_str(&format!(
            "    let {snake} = pick(&[\"{env}\"], {order}, shell, dotenv, flags, {default});\n",
            snake = flag.snake,
            env = rust_escape(&flag.env),
            order = order,
            default = default,
        ));
        if flag.required {
            let examples = rust_examples(flag);
            out.push_str(&format!(
                "    let {snake} = require_env(\"{env}\", \"{ty}\", {examples}, {snake})?;\n",
                snake = flag.snake,
                env = rust_escape(&flag.env),
                ty = flag.flag_type,
                examples = examples,
            ));
            out.push_str(&format!(
                "    out.insert(\"{}\".to_string(), {snake});\n",
                rust_escape(&flag.env),
                snake = flag.snake
            ));
        } else {
            out.push_str(&format!(
                "    if let Some(value) = {snake} {{\n        out.insert(\"{}\".to_string(), value);\n    }}\n",
                rust_escape(&flag.env),
                snake = flag.snake
            ));
        }
    }
    out.push_str(
        "    match check_os_env(&out) {\n        Ok(()) => Ok(out),\n        Err(errors) => Err(contract_error_to_missing(&errors[0])),\n    }\n}\n\n",
    );
    out.push_str(
        "/// Effectful overlay: `.env` files then the process environment, ranked per key.\n",
    );
    out.push_str(&format!(
        "pub fn load_env_map_from_os() -> Result<std::collections::BTreeMap<String, String>, MissingEnv> {{\n    load_env_map(&shell_env(), &load_dotenv_files(&{}), &std::collections::BTreeMap::new())\n}}\n",
        rust_files_array(&catalog.env_load.files, catalog.env_load.load)
    ));
    out
}

pub fn render_typescript(catalog: &Catalog) -> String {
    let mut out = String::from(TS_HELPERS);
    out.push_str("export interface MissingEnv {\n  readonly envKey: string;\n  readonly expectedType: string;\n  readonly examples: readonly string[];\n}\n\n");
    out.push_str("export class MissingEnvError extends Error implements MissingEnv {\n");
    out.push_str("  readonly envKey: string;\n  readonly expectedType: string;\n  readonly examples: readonly string[];\n");
    out.push_str("  constructor(fields: MissingEnv) {\n    super(`missing required environment variable ${fields.envKey}\\n  expected type: ${fields.expectedType}\\n  examples: ${fields.examples.join(\", \")}`);\n");
    out.push_str("    this.name = \"MissingEnvError\";\n    this.envKey = fields.envKey;\n    this.expectedType = fields.expectedType;\n    this.examples = fields.examples;\n  }\n}\n\n");
    out.push_str(
        "/** Resolve env-key -> value. Empty values fall through to the next source. */\n",
    );
    out.push_str("export function loadEnvMap(\n  shell: Record<string, string | undefined>,\n  dotenv: Record<string, string | undefined>,\n  flags: Record<string, string | undefined> = {},\n): Record<string, string> {\n  const out: Record<string, string> = {};\n");
    for flag in &catalog.flags {
        let order = ts_order_array(&catalog.source_order_for(flag));
        let default = match &flag.default {
            Some(value) => format!("\"{}\"", ts_escape(value)),
            None => "undefined".into(),
        };
        out.push_str(&format!(
            "  const {snake} = pick([\"{env}\"], {order}, shell, dotenv, flags, {default});\n",
            snake = flag.snake,
            env = ts_escape(&flag.env),
            order = order,
            default = default,
        ));
        if flag.required {
            out.push_str(&format!(
                "  out[\"{env}\"] = requireEnv(\"{env}\", \"{ty}\", {examples}, {snake});\n",
                env = ts_escape(&flag.env),
                ty = flag.flag_type,
                examples = ts_examples(flag),
                snake = flag.snake,
            ));
        } else {
            out.push_str(&format!(
                "  if ({snake} !== undefined) out[\"{}\"] = {snake};\n",
                ts_escape(&flag.env),
                snake = flag.snake
            ));
        }
    }
    out.push_str(
        "  const contract = checkOsEnv(out);\n  if (contract.length > 0) {\n    const first = contract[0];\n    throw new MissingEnvError({ envKey: first.path, expectedType: \"json-schema-2020-12\", examples: [] });\n  }\n  return out;\n}\n\n",
    );
    out.push_str(&format!(
        "const DOTENV_FILES: readonly string[] = {};\n",
        ts_files_array(&catalog.env_load.files, catalog.env_load.load)
    ));
    out.push_str("/** Effectful overlay: `.env` files then `process.env`, ranked per key. */\n");
    out.push_str("export function loadEnvMapFromOs(\n  shell: Record<string, string | undefined> = typeof process !== \"undefined\" ? process.env : {},\n): Record<string, string> {\n  return loadEnvMap(shell, loadDotenvFiles(DOTENV_FILES), {});\n}\n");
    out
}

pub fn render_dart(catalog: &Catalog) -> String {
    let mut out = String::from(DART_HELPERS);
    out.push_str("final class MissingEnv implements Exception {\n  const MissingEnv({required this.name, required this.expectedType, required this.examples});\n");
    out.push_str(
        "  final String name;\n  final String expectedType;\n  final List<String> examples;\n",
    );
    out.push_str("  @override\n  String toString() => 'missing required environment variable $name\\n  expected type: $expectedType\\n  examples: ${examples.join(', ')}';\n}\n\n");
    out.push_str("Map<String, String> loadEnvMap(\n  Map<String, String> shell,\n  Map<String, String> dotenv, [\n  Map<String, String> flags = const {},\n]) {\n  final out = <String, String>{};\n");
    for flag in &catalog.flags {
        let order = dart_order_array(&catalog.source_order_for(flag));
        let default = match &flag.default {
            Some(value) => format!("'{}'", dart_escape(value)),
            None => "null".into(),
        };
        out.push_str(&format!(
            "  final {ident} = pick(['{env}'], {order}, shell, dotenv, flags, {default});\n",
            ident = dart_ident(&flag.snake),
            env = dart_escape(&flag.env),
            order = order,
            default = default,
        ));
        if flag.required {
            out.push_str(&format!(
                "  out['{env}'] = requireEnv('{env}', '{ty}', {examples}, {ident});\n",
                env = dart_escape(&flag.env),
                ty = flag.flag_type,
                examples = dart_examples(flag),
                ident = dart_ident(&flag.snake),
            ));
        } else {
            out.push_str(&format!(
                "  if ({ident} != null) out['{}'] = {ident};\n",
                dart_escape(&flag.env),
                ident = dart_ident(&flag.snake)
            ));
        }
    }
    out.push_str(
        "  final contract = checkOsEnv(out);\n  if (contract.isNotEmpty) {\n    throw MissingEnv(name: contract.first.path, expectedType: 'json-schema-2020-12', examples: const <String>[]);\n  }\n  return out;\n}\n\n",
    );
    out.push_str(&format!(
        "const List<String> _dotenvFiles = {};\n\n",
        dart_files_array(&catalog.env_load.files, catalog.env_load.load)
    ));
    out.push_str("Map<String, String> loadEnvMapFromOs() {\n  return loadEnvMap(platform.osEnvironment(), loadDotenvFiles(_dotenvFiles));\n}\n");
    out
}

fn rust_order_array(order: &[String]) -> String {
    format!(
        "&[{}]",
        order
            .iter()
            .map(|source| format!("\"{source}\""))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn rust_files_array(files: &[String], load: bool) -> String {
    if !load || files.is_empty() {
        return "[]".into();
    }
    format!(
        "[{}]",
        files
            .iter()
            .map(|path| format!("\"{}\"", rust_escape(path)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn rust_examples(flag: &FlagSpec) -> String {
    format!(
        "&[{}]",
        example_values(flag)
            .iter()
            .map(|value| format!("\"{}\"", rust_escape(value)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn ts_order_array(order: &[String]) -> String {
    format!(
        "[{}]",
        order
            .iter()
            .map(|source| format!("\"{source}\""))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn ts_files_array(files: &[String], load: bool) -> String {
    if !load || files.is_empty() {
        return "[]".into();
    }
    format!(
        "[{}]",
        files
            .iter()
            .map(|path| format!("\"{}\"", ts_escape(path)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn ts_examples(flag: &FlagSpec) -> String {
    format!(
        "[{}]",
        example_values(flag)
            .iter()
            .map(|value| format!("\"{}\"", ts_escape(value)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn dart_order_array(order: &[String]) -> String {
    format!(
        "[{}]",
        order
            .iter()
            .map(|source| format!("'{source}'"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn dart_files_array(files: &[String], load: bool) -> String {
    if !load || files.is_empty() {
        return "[]".into();
    }
    format!(
        "[{}]",
        files
            .iter()
            .map(|path| format!("'{}'", dart_escape(path)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn dart_examples(flag: &FlagSpec) -> String {
    format!(
        "[{}]",
        example_values(flag)
            .iter()
            .map(|value| format!("'{}'", dart_escape(value)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn dart_ident(snake: &str) -> String {
    let mut parts = snake.split('_');
    let Some(first) = parts.next() else {
        return "value".into();
    };
    let mut out = first.to_string();
    for part in parts {
        if part.is_empty() {
            continue;
        }
        let mut chars = part.chars();
        if let Some(head) = chars.next() {
            out.push(head.to_ascii_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
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

const RUST_HELPERS: &str = r#"
fn nonempty(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim).filter(|value| !value.is_empty()).map(str::to_string)
}

fn require_env(
    name: &'static str,
    expected_type: &'static str,
    examples: &'static [&'static str],
    value: Option<String>,
) -> Result<String, MissingEnv> {
    match nonempty(value.as_deref()) {
        Some(value) => Ok(value),
        None => Err(MissingEnv {
            name,
            expected_type,
            examples,
        }),
    }
}

fn pick(
    keys: &[&str],
    order: &[&str],
    shell: &std::collections::BTreeMap<String, String>,
    dotenv: &std::collections::BTreeMap<String, String>,
    flags: &std::collections::BTreeMap<String, String>,
    default: Option<&str>,
) -> Option<String> {
    for source in order {
        let map = match *source {
            "flags" => flags,
            "env_file" => dotenv,
            _ => shell,
        };
        for key in keys {
            if let Some(value) = nonempty(map.get(*key).map(String::as_str)) {
                return Some(value);
            }
        }
    }
    nonempty(default)
}

fn unquote(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        if (bytes[0] == b'"' && bytes[trimmed.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[trimmed.len() - 1] == b'\'')
        {
            return trimmed[1..trimmed.len() - 1].to_string();
        }
    }
    trimmed.to_string()
}

fn parse_dotenv(text: &str) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").map(str::trim).unwrap_or(line);
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if !is_dotenv_key(key) {
            continue;
        }
        out.insert(key.to_string(), unquote(value));
    }
    out
}

fn is_dotenv_key(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {
            chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        }
        _ => false,
    }
}

fn is_safe_dotenv_path(path: &str) -> bool {
    if path.is_empty() || path.contains('\0') {
        return false;
    }
    let parsed = std::path::Path::new(path);
    if parsed.is_absolute() {
        return false;
    }
    if parsed
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return false;
    }
    matches!(
        parsed.file_name().and_then(|name| name.to_str()),
        Some(name) if name == ".env" || name.starts_with(".env.")
    )
}

fn dotenv_enabled() -> bool {
    !matches!(
        std::env::var("FLAGS2ENV_DOTENV"),
        Ok(value) if matches!(value.trim(), "0" | "false" | "FALSE" | "no" | "NO")
    )
}

fn load_dotenv_files(files: &[&str]) -> std::collections::BTreeMap<String, String> {
    if !dotenv_enabled() {
        return std::collections::BTreeMap::new();
    }
    files.iter().filter(|path| is_safe_dotenv_path(path)).fold(
        std::collections::BTreeMap::new(),
        |mut acc, path| {
            if let Ok(text) = std::fs::read_to_string(path) {
                acc.extend(parse_dotenv(&text));
            }
            acc
        },
    )
}

fn shell_env() -> std::collections::BTreeMap<String, String> {
    std::env::vars().collect()
}
"#;

const TS_HELPERS: &str = r##"
function nonempty(raw: string | undefined): string | undefined {
  const value = raw?.trim();
  return value ? value : undefined;
}

export function requireEnv(
  name: string,
  expectedType: string,
  examples: readonly string[],
  value: string | undefined,
): string {
  const trimmed = nonempty(value);
  if (trimmed) {
    return trimmed;
  }
  throw new MissingEnvError({ envKey: name, expectedType, examples });
}

function pick(
  keys: readonly string[],
  order: readonly string[],
  shell: Record<string, string | undefined>,
  dotenv: Record<string, string | undefined>,
  flags: Record<string, string | undefined>,
  fallback: string | undefined,
): string | undefined {
  for (const source of order) {
    const map = source === "flags" ? flags : source === "env_file" ? dotenv : shell;
    for (const key of keys) {
      const value = nonempty(map[key]);
      if (value !== undefined) {
        return value;
      }
    }
  }
  return nonempty(fallback);
}

function unquote(value: string): string {
  const trimmed = value.trim();
  if (
    (trimmed.startsWith("\"") && trimmed.endsWith("\"")) ||
    (trimmed.startsWith("'") && trimmed.endsWith("'"))
  ) {
    return trimmed.slice(1, -1);
  }
  return trimmed;
}

export function parseDotenv(text: string): Record<string, string> {
  const out: Record<string, string> = {};
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line || line.startsWith("#")) {
      continue;
    }
    const body = line.startsWith("export ") ? line.slice("export ".length).trim() : line;
    const eq = body.indexOf("=");
    if (eq <= 0) {
      continue;
    }
    const key = body.slice(0, eq).trim();
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(key)) {
      continue;
    }
    out[key] = unquote(body.slice(eq + 1));
  }
  return out;
}

function dotenvEnabled(): boolean {
  const value = typeof process !== "undefined" ? process.env.FLAGS2ENV_DOTENV : undefined;
  return !["0", "false", "FALSE", "no", "NO"].includes(value?.trim() ?? "");
}

function isSafeDotenvPath(path: string): boolean {
  if (!path || path.includes("\0") || path.includes("..")) {
    return false;
  }
  if (path.startsWith("/") || /^[A-Za-z]:[\\/]/.test(path)) {
    return false;
  }
  const base = path.split(/[\\/]/).pop() ?? "";
  return base === ".env" || base.startsWith(".env.");
}

export function loadDotenvFiles(files: readonly string[]): Record<string, string> {
  if (!dotenvEnabled() || typeof process === "undefined") {
    return {};
  }
  let fs: { readFileSync: (path: string, encoding: string) => string } | undefined;
  try {
    fs = require("fs") as { readFileSync: (path: string, encoding: string) => string };
  } catch {
    return {};
  }
  return files.filter(isSafeDotenvPath).reduce<Record<string, string>>((acc, path) => {
    try {
      return { ...acc, ...parseDotenv(fs!.readFileSync(path, "utf8")) };
    } catch {
      return acc;
    }
  }, {});
}
"##;

const DART_HELPERS: &str = r#"
String? _nonempty(String? raw) {
  final value = raw?.trim();
  if (value == null || value.isEmpty) {
    return null;
  }
  return value;
}

String requireEnv(String name, String expectedType, List<String> examples, String? value) {
  final trimmed = _nonempty(value);
  if (trimmed != null) {
    return trimmed;
  }
  throw MissingEnv(name: name, expectedType: expectedType, examples: examples);
}

String? pick(
  List<String> keys,
  List<String> order,
  Map<String, String> shell,
  Map<String, String> dotenv,
  Map<String, String> flags,
  String? fallback,
) {
  for (final source in order) {
    final map = source == 'flags' ? flags : source == 'env_file' ? dotenv : shell;
    for (final key in keys) {
      final value = _nonempty(map[key]);
      if (value != null) {
        return value;
      }
    }
  }
  return _nonempty(fallback);
}

String _unquote(String value) {
  final trimmed = value.trim();
  if (trimmed.length >= 2 &&
      ((trimmed.startsWith('"') && trimmed.endsWith('"')) ||
          (trimmed.startsWith("'") && trimmed.endsWith("'")))) {
    return trimmed.substring(1, trimmed.length - 1);
  }
  return trimmed;
}

Map<String, String> parseDotenv(String text) {
  final out = <String, String>{};
  for (final raw in text.split('\n')) {
    var line = raw.trim();
    if (line.isEmpty || line.startsWith('#')) {
      continue;
    }
    if (line.startsWith('export ')) {
      line = line.substring('export '.length).trim();
    }
    final eq = line.indexOf('=');
    if (eq <= 0) {
      continue;
    }
    final key = line.substring(0, eq).trim();
    if (!RegExp(r'^[A-Za-z_][A-Za-z0-9_]*$').hasMatch(key)) {
      continue;
    }
    out[key] = _unquote(line.substring(eq + 1));
  }
  return out;
}

bool _dotenvEnabled() {
  final value = platform.osEnvironment()['FLAGS2ENV_DOTENV'];
  return value != '0' && value != 'false' && value != 'FALSE' && value != 'no' && value != 'NO';
}

bool _isSafeDotenvPath(String path) {
  if (path.isEmpty || path.contains('\0') || path.contains('..')) {
    return false;
  }
  if (path.startsWith('/') || RegExp(r'^[A-Za-z]:[\\/]').hasMatch(path)) {
    return false;
  }
  final segments = path.split(RegExp(r'[\\/]'));
  if (segments.contains('..')) {
    return false;
  }
  final base = segments.isEmpty ? '' : segments.last;
  return base == '.env' || base.startsWith('.env.');
}

Map<String, String> loadDotenvFiles(List<String> files) {
  if (!_dotenvEnabled()) {
    return {};
  }
  return files.where(_isSafeDotenvPath).fold<Map<String, String>>({}, (acc, path) {
    final text = platform.readFileUtf8(path);
    if (text != null) {
      acc.addAll(parseDotenv(text));
    }
    return acc;
  });
}
"#;

#[cfg(test)]
mod tests {
    use crate::catalog::parse_catalog;

    #[test]
    fn overlay_defaults_to_no_dotenv_files() {
        let catalog = parse_catalog(
            r#"
[flags.bind]
env = "APP_BIND"
default = "127.0.0.1:8080"
"#,
            None,
        )
        .unwrap();
        let rust = super::render_rust(&catalog);
        assert!(rust.contains("load_dotenv_files(&[])"));
        assert!(rust.contains("fn is_safe_dotenv_path"));
        let ts = super::render_typescript(&catalog);
        assert!(ts.contains("this.name = \"MissingEnvError\""));
        assert!(ts.contains("readonly envKey: string"));
        assert!(!ts.contains("this.name = fields.name"));
        let dart = super::render_dart(&catalog);
        assert!(dart.contains("platform.osEnvironment()"));
        assert!(dart.contains("_isSafeDotenvPath"));
        assert!(!dart.contains("Platform.environment"));
        assert!(!dart.contains("File(path)"));
    }
}
