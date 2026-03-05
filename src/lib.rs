//! OmniLang - The Ultimate Programming Language
//! 
//! A unified programming language combining the best features of Python, C++, Java, Rust,
//! while introducing novel capabilities for systems programming, web development,
//! data science, AI/ML, and embedded systems.

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod typecheck;
pub mod codegen;
pub mod llvm;
pub mod stdlib;
pub mod errors;
pub mod tools;
pub mod diagnostics;
pub mod lsp;
pub mod package_manager;

pub use errors::{Result, OmniError};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "OmniLang";
