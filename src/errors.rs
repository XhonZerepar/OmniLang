//! Error handling for OmniLang

use thiserror::Error;

pub type Result<T> = std::result::Result<T, OmniError>;

#[derive(Error, Debug)]
pub enum OmniError {
    #[error("Lexer error at {location}: {message}")]
    LexerError {
        location: String,
        message: String,
    },

    #[error("Parser error at {location}: {message}")]
    ParserError {
        location: String,
        message: String,
    },

    #[error("Type error at {location}: {message}")]
    TypeError {
        location: String,
        message: String,
    },

    #[error("Code generation error: {message}")]
    CodegenError {
        message: String,
    },

    #[error("IO error: {message}")]
    IoError {
        message: String,
    },

    #[error("Compilation error: {message}")]
    CompilationError {
        message: String,
    },

    #[error("Runtime error: {message}")]
    RuntimeError {
        message: String,
    },
}

impl From<std::io::Error> for OmniError {
    fn from(err: std::io::Error) -> Self {
        OmniError::IoError {
            message: err.to_string(),
        }
    }
}

impl From<inkwell::llvm::Error> for OmniError {
    fn from(err: inkwell::llvm::Error) -> Self {
        OmniError::CodegenError {
            message: err.to_string(),
        }
    }
}
