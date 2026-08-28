#![forbid(unsafe_code)]

mod dart;
mod gleam;
mod overlay;
mod rust;
mod scaffold;
mod typescript;

use crate::args::Language;
use crate::catalog::Catalog;

pub fn render(language: Language, catalog: &Catalog) -> String {
    match language {
        Language::Rust => rust::render(catalog),
        Language::Dart => dart::render(catalog),
        Language::TypeScript => typescript::render(catalog),
        Language::Gleam => gleam::render(catalog),
    }
}

pub fn render_runtime(language: Language, catalog: &Catalog) -> String {
    match language {
        Language::Rust => rust::render_runtime(catalog),
        Language::Dart => dart::render_runtime(catalog),
        Language::TypeScript => typescript::render_runtime(catalog),
        Language::Gleam => gleam::render_runtime(catalog),
    }
}

pub fn file_name(language: Language) -> &'static str {
    match language {
        Language::Rust => "env.rs",
        Language::Dart => "env.dart",
        Language::TypeScript => "env.ts",
        Language::Gleam => "env.gleam",
    }
}

pub fn runtime_file_name(language: Language) -> &'static str {
    match language {
        Language::Rust => "runtime.rs",
        Language::Dart => "runtime.dart",
        Language::TypeScript => "runtime.ts",
        Language::Gleam => "runtime.gleam",
    }
}

pub fn render_overlay(language: Language, catalog: &Catalog) -> Option<String> {
    match language {
        Language::Rust => Some(overlay::render_rust(catalog)),
        Language::Dart => Some(overlay::render_dart(catalog)),
        Language::TypeScript => Some(overlay::render_typescript(catalog)),
        Language::Gleam => None,
    }
}

pub fn scaffold_readme() -> &'static str {
    scaffold::readme()
}

pub fn scaffold_mod_rs() -> &'static str {
    scaffold::rust_mod()
}

pub fn scaffold_env(language: Language, catalog: &Catalog) -> Option<String> {
    match language {
        Language::Rust => Some(scaffold::rust_env(catalog)),
        Language::Dart => Some(scaffold::dart_env(catalog)),
        Language::TypeScript => Some(scaffold::typescript_env(catalog)),
        Language::Gleam => None,
    }
}

pub fn generated_flat_name(language: Language) -> &'static str {
    match language {
        Language::Rust => "generated.rs",
        Language::Dart => "generated.dart",
        Language::TypeScript => "generated.ts",
        Language::Gleam => "generated.gleam",
    }
}

pub fn dir_name(language: Language) -> &'static str {
    match language {
        Language::Rust => "rust",
        Language::Dart => "dart",
        Language::TypeScript => "typescript",
        Language::Gleam => "gleam",
    }
}
