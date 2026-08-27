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

pub fn file_name(language: Language) -> &'static str {
    match language {
        Language::Rust => "env.rs",
        Language::Dart => "env.dart",
        Language::TypeScript => "env.ts",
        Language::Gleam => "env.gleam",
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
