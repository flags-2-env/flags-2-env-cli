#![forbid(unsafe_code)]

mod dart;
mod gleam;
mod rust;
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

pub fn dir_name(language: Language) -> &'static str {
    match language {
        Language::Rust => "rust",
        Language::Dart => "dart",
        Language::TypeScript => "typescript",
        Language::Gleam => "gleam",
    }
}
