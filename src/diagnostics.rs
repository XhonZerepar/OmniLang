//! Improved Error Handling for OmniLang
//! 
//! Provides beautiful, colorful error messages with source spans.

use crate::ast::{Position, Span};
use std::fmt;
use std::ops::Range;

/// Source code location with span information
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub filename: String,
    pub line: usize,
    pub column: usize,
    pub span: Option<Range<usize>>,
}

impl SourceLocation {
    pub fn new(filename: String, line: usize, column: usize) -> Self {
        Self {
            filename,
            line,
            column,
            span: None,
        }
    }

    pub fn with_span(mut self, span: Range<usize>) -> Self {
        self.span = Some(span);
        self
    }

    pub fn from_position(pos: &Position) -> Self {
        Self {
            filename: pos.filename.clone(),
            line: pos.line,
            column: pos.column,
            span: None,
        }
    }

    pub fn from_span(span: &Span) -> Self {
        Self {
            filename: span.start.filename.clone(),
            line: span.start.line,
            column: span.start.column,
            span: Some(span.start.column..span.end.column),
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.filename, self.line, self.column)
    }
}

/// Severity level for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}

/// Diagnostic message with source location
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub location: SourceLocation,
    pub severity: Severity,
    pub labels: Vec<Label>,
    pub help: Option<String>,
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn error(code: impl Into<String>, message: impl Into<String>, location: SourceLocation) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            location,
            severity: Severity::Error,
            labels: Vec::new(),
            help: None,
            notes: Vec::new(),
        }
    }

    pub fn warning(code: impl Into<String>, message: impl Into<String>, location: SourceLocation) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            location,
            severity: Severity::Warning,
            labels: Vec::new(),
            help: None,
            notes: Vec::new(),
        }
    }

    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Render the diagnostic as a colorful string
    pub fn render(&self, source: &str) -> String {
        let mut output = String::new();
        
        // Color codes
        let red = "\x1b[91m";
        let yellow = "\x1b[93m";
        let blue = "\x1b[94m";
        let green = "\x1b[92m";
        let cyan = "\x1b[96m";
        let bold = "\x1b[1m";
        let reset = "\x1b[0m";
        
        // Severity indicator
        let (severity_str, severity_color) = match self.severity {
            Severity::Error => ("error", red),
            Severity::Warning => ("warning", yellow),
            Severity::Note => ("note", blue),
            Severity::Help => ("help", green),
        };
        
        // Error code header
        output.push_str(&format!(
            "{}{}{}{}: {} {}{}{}\n",
            bold, severity_color, severity_str, reset,
            self.code,
            cyan, self.location, reset
        ));
        
        // Main message
        output.push_str(&format!("  {}{}{}\n", bold, self.message, reset));
        
        // Source code display
        if let Some(span) = &self.location.span {
            let lines: Vec<&str> = source.lines().collect();
            if self.location.line > 0 && self.location.line <= lines.len() {
                let line_num = self.location.line - 1;
                let line = lines[line_num];
                
                // Line number
                let line_str = format!("{:>4} │ ", self.location.line);
                output.push_str(&format!("{} {}{}\n", cyan, line_str, reset));
                
                // Pointer to error
                let pointer = "      │ ";
                output.push_str(pointer);
                
                // Add spaces up to the error column
                let spaces = self.location.column.saturating_sub(1);
                for _ in 0..spaces {
                    output.push(' ');
                }
                
                // Add the error marker
                let error_len = span.end.saturating_sub(span.start);
                let error_len = if error_len == 0 { 1 } else { error_len };
                output.push_str(&format!("{}{}{}", red, "^".repeat(error_len), reset));
                
                // Show what it should be
                if let Some(help) = &self.help {
                    output.push_str(&format!(" {}{}{}", blue, help, reset));
                }
                
                output.push('\n');
            }
        }
        
        // Additional labels
        for label in &self.labels {
            output.push_str(&format!(
                "  {} --> {}{}:{}:{}{}\n",
                blue, cyan, label.location.filename, 
                label.location.line, label.location.column, reset
            ));
            output.push_str(&format!(
                "    {}\n",
                label.message
            ));
        }
        
        // Help message
        if let Some(help) = &self.help {
            output.push_str(&format!("  {} = Help: {}{}\n", green, help, reset));
        }
        
        // Notes
        for note in &self.notes {
            output.push_str(&format!("  {} = Note: {}{}\n", blue, note, reset));
        }
        
        output
    }
}

/// Label for additional source context
#[derive(Debug, Clone)]
pub struct Label {
    pub message: String,
    pub location: SourceLocation,
}

impl Label {
    pub fn new(message: impl Into<String>, location: SourceLocation) -> Self {
        Self {
            message: message.into(),
            location,
        }
    }
}

/// Collection of diagnostics
#[derive(Debug, Default)]
pub struct Diagnostics {
    pub diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn error(&mut self, code: impl Into<String>, message: impl Into<String>, location: SourceLocation) {
        self.push(Diagnostic::error(code, message, location));
    }

    pub fn warning(&mut self, code: impl Into<String>, message: impl Into<String>, location: SourceLocation) {
        self.push(Diagnostic::warning(code, message, location));
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn render_all(&self, source: &str) -> String {
        self.diagnostics
            .iter()
            .map(|d| d.render(source))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn count(&self) -> (usize, usize) {
        let errors = self.diagnostics.iter().filter(|d| d.severity == Severity::Error).count();
        let warnings = self.diagnostics.iter().filter(|d| d.severity == Severity::Warning).count();
        (errors, warnings)
    }
}

/// Error codes for OmniLang
pub mod codes {
    pub const LEXER001: &str = "E001";
    pub const LEXER002: &str = "E002";
    pub const PARSER001: &str = "P001";
    pub const PARSER002: &str = "P002";
    pub const PARSER003: &str = "P003";
    pub const TYPE001: &str = "T001";
    pub const TYPE002: &str = "T002";
    pub const TYPE003: &str = "T003";
    pub const COMPILE001: &str = "C001";
    pub const COMPILE002: &str = "C002";
}
