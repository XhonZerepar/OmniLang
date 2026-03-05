//! OmniLang Language Server (omlsp)
//! 
//! Provides IDE support via Language Server Protocol

use crate::lexer::Lexer;
use crate::parser::Parser as OmniParser;
use crate::ast::{Program, Span, Position};
use std::io::{Read, Write};
use std::process::{Stdio, Child};
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

/// Language Server Protocol types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum Request {
    #[serde(rename = "initialize")]
    Initialize { params: InitializeParams },
    
    #[serde(rename = "textDocument/didOpen")]
    DidOpen { params: DidOpenTextDocumentParams },
    
    #[serde(rename = "textDocument/didChange")]
    DidChange { params: DidChangeTextDocumentParams },
    
    #[serde(rename = "textDocument/didSave")]
    DidSave { params: DidSaveTextDocumentParams },
    
    #[serde(rename = "textDocument/didClose")]
    DidClose { params: DidCloseTextDocumentParams },
    
    #[serde(rename = "textDocument/hover")]
    Hover { params: TextDocumentPositionParams },
    
    #[serde(rename = "textDocument/definition")]
    Definition { params: TextDocumentPositionParams },
    
    #[serde(rename = "textDocument/completion")]
    Completion { params: CompletionParams },
    
    #[serde(rename = "textDocument/documentSymbol")]
    DocumentSymbol { params: DocumentSymbolParams },
    
    #[serde(rename = "textDocument/codeAction")]
    CodeAction { params: CodeActionParams },
    
    #[serde(rename = "shutdown")]
    Shutdown,
    
    #[serde(rename = "exit")]
    Exit,
}

/// Initialize request parameters
#[derive(Debug, Clone, Deserialize)]
pub struct InitializeParams {
    pub process_id: Option<usize>,
    pub root_uri: Option<String>,
    pub capabilities: ClientCapabilities,
}

/// Client capabilities
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ClientCapabilities {
    pub text_document: Option<TextDocumentClientCapabilities>,
    pub workspace: Option<WorkspaceClientCapabilities>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TextDocumentClientCapabilities {
    pub synchronization: Option<SynchronizationCapability>,
    pub completion: Option<CompletionCapability>,
    pub hover: Option<HoverCapability>,
    pub definition: Option<DefinitionCapability>,
    pub references: Option<ReferenceCapability>,
    pub document_symbol: Option<DocumentSymbolCapability>,
    pub code_action: Option<CodeActionCapability>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WorkspaceClientCapabilities {
    pub apply_edit: Option<bool>,
    pub workspace_folders: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SynchronizationCapability {
    pub dynamic_registration: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CompletionCapability {
    pub dynamic_registration: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct HoverCapability {
    pub dynamic_registration: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DefinitionCapability {
    pub dynamic_registration: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ReferenceCapability {
    pub dynamic_registration: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DocumentSymbolCapability {
    pub dynamic_registration: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CodeActionCapability {
    pub dynamic_registration: Option<bool>,
}

/// Text document document
#[derive(Debug, Clone)]
pub struct TextDocument {
    pub uri: String,
    pub text: String,
    pub version: i32,
    pub ast: Option<Program>,
}

/// Server capabilities
#[derive(Debug, Clone, Serialize)]
pub struct ServerCapabilities {
    pub text_document_sync: TextDocumentSyncOptions,
    pub hover_provider: bool,
    pub definition_provider: bool,
    pub references_provider: bool,
    pub completion_provider: CompletionOptions,
    pub document_symbol_provider: bool,
    pub code_action_provider: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TextDocumentSyncOptions {
    pub open_close: bool,
    pub change: i32, // 1 = Full, 2 = Incremental
    pub save: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletionOptions {
    pub resolve_provider: bool,
    pub trigger_characters: Vec<String>,
}

/// Did open text document notification
#[derive(Debug, Clone, Deserialize)]
pub struct DidOpenTextDocumentParams {
    pub text_document: TextDocumentItem,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextDocumentItem {
    pub uri: String,
    pub language_id: String,
    pub version: i32,
    pub text: String,
}

/// Did change text document notification
#[derive(Debug, Clone, Deserialize)]
pub struct DidChangeTextDocumentParams {
    pub text_document: VersionedTextDocumentIdentifier,
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextDocumentContentChangeEvent {
    pub text: Option<String>,
    pub range: Option<Range>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DidSaveTextDocumentParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DidCloseTextDocumentParams {
    pub text_document: TextDocumentIdentifier,
}

/// Text document position parameters
#[derive(Debug, Clone, Deserialize)]
pub struct TextDocumentPositionParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

/// Completion parameters
#[derive(Debug, Clone, Deserialize)]
pub struct CompletionParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
    pub context: Option<CompletionContext>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionContext {
    pub trigger_kind: i32,
    pub trigger_character: Option<String>,
}

/// Document symbol parameters
#[derive(Debug, Clone, Deserialize)]
pub struct DocumentSymbolParams {
    pub text_document: TextDocumentIdentifier,
}

/// Code action parameters
#[derive(Debug, Clone, Deserialize)]
pub struct CodeActionParams {
    pub text_document: TextDocumentIdentifier,
    pub range: Range,
    pub context: CodeActionContext,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CodeActionContext {
    pub diagnostics: Vec<Diagnostic>,
}

/// LSP Position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

/// LSP Range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: LspPosition,
    pub end: LspPosition,
}

/// LSP Location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

/// Diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: i32,
    pub message: String,
    pub code: Option<String>,
}

/// Hover response
#[derive(Debug, Clone, Serialize)]
pub struct Hover {
    pub contents: MarkedString,
    pub range: Option<Range>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum MarkedString {
    String(String),
    Markup { language: String, value: String },
}

/// Completion item
#[derive(Debug, Clone, Serialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: i32,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
}

/// Document symbol
#[derive(Debug, Clone, Serialize)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: i32,
    pub range: Range,
    pub children: Vec<DocumentSymbol>,
}

/// Language server state
pub struct LanguageServer {
    documents: Mutex<std::collections::HashMap<String, TextDocument>>,
    capabilities: ServerCapabilities,
}

impl LanguageServer {
    pub fn new() -> Self {
        Self {
            documents: Mutex::new(std::collections::HashMap::new()),
            capabilities: ServerCapabilities {
                text_document_sync: TextDocumentSyncOptions {
                    open_close: true,
                    change: 2,
                    save: true,
                },
                hover_provider: true,
                definition_provider: true,
                references_provider: true,
                completion_provider: CompletionOptions {
                    resolve_provider: false,
                    trigger_characters: vec![".".to_string(), ":".to_string()],
                },
                document_symbol_provider: true,
                code_action_provider: true,
            },
        }
    }
    
    /// Handle textDocument/didOpen
    pub fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = TextDocument {
            uri: params.text_document.uri.clone(),
            text: params.text_document.text.clone(),
            version: params.text_document.version,
            ast: None,
        };
        
        // Parse and store AST
        let ast = self.parse_text(&doc.text, &doc.uri);
        
        let mut docs = self.documents.lock().unwrap();
        docs.insert(params.text_document.uri, TextDocument {
            uri: doc.uri,
            text: doc.text,
            version: doc.version,
            ast,
        });
    }
    
    /// Handle textDocument/didChange
    pub fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut docs = self.documents.lock().unwrap();
        
        if let Some(doc) = docs.get_mut(&params.text_document.uri) {
            for change in params.content_changes {
                if let Some(text) = change.text {
                    doc.text = text;
                    doc.version = params.text_document.version;
                    doc.ast = Some(self.parse_text(&doc.text, &doc.uri));
                }
            }
        }
    }
    
    /// Handle textDocument/didSave
    pub fn did_save(&self, _params: DidSaveTextDocumentParams) {
        // Could trigger compilation or other actions
    }
    
    /// Handle textDocument/didClose
    pub fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut docs = self.documents.lock().unwrap();
        docs.remove(&params.text_document.uri);
    }
    
    /// Handle textDocument/hover
    pub fn hover(&self, params: TextDocumentPositionParams) -> Option<Hover> {
        let docs = self.documents.lock().unwrap();
        
        if let Some(doc) = docs.get(&params.text_document.uri) {
            let position = params.position;
            let offset = self.position_to_offset(&doc.text, position.line, position.character);
            
            if let Some(ast) = &doc.ast {
                // Find symbol at position
                // For now, return a basic hover
                return Some(Hover {
                    contents: MarkedString::String("OmniLang code".to_string()),
                    range: None,
                });
            }
        }
        
        None
    }
    
    /// Handle textDocument/definition
    pub fn definition(&self, params: TextDocumentPositionParams) -> Option<Location> {
        let docs = self.documents.lock().unwrap();
        
        if let Some(doc) = docs.get(&params.text_document.uri) {
            let position = params.position;
            // Find definition location
            // This would require symbol table integration
        }
        
        None
    }
    
    /// Handle textDocument/completion
    pub fn completion(&self, _params: CompletionParams) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "fn".to_string(),
                kind: 14, // Keyword
                detail: Some("Function declaration".to_string()),
                documentation: None,
                insert_text: Some("fn name(args) -> ReturnType:\n    ".to_string()),
            },
            CompletionItem {
                label: "let".to_string(),
                kind: 14,
                detail: Some("Immutable variable".to_string()),
                documentation: None,
                insert_text: Some("let name = value".to_string()),
            },
            CompletionItem {
                label: "mut".to_string(),
                kind: 14,
                detail: Some("Mutable variable".to_string()),
                documentation: None,
                insert_text: Some("mut name = value".to_string()),
            },
            CompletionItem {
                label: "struct".to_string(),
                kind: 14,
                detail: Some("Struct definition".to_string()),
                documentation: None,
                insert_text: Some("struct Name:\n    field: Type".to_string()),
            },
            CompletionItem {
                label: "if".to_string(),
                kind: 14,
                detail: Some("Conditional".to_string()),
                documentation: None,
                insert_text: Some("if condition:\n    ".to_string()),
            },
            CompletionItem {
                label: "for".to_string(),
                kind: 14,
                detail: Some("For loop".to_string()),
                documentation: None,
                insert_text: Some("for item in collection:\n    ".to_string()),
            },
            CompletionItem {
                label: "match".to_string(),
                kind: 14,
                detail: Some("Pattern matching".to_string()),
                documentation: None,
                insert_text: Some("match value:\n    pattern => result".to_string()),
            },
            CompletionItem {
                label: "println".to_string(),
                kind: 6, // Function
                detail: Some("Print with newline".to_string()),
                documentation: None,
                insert_text: Some("println(\"message\")".to_string()),
            },
        ]
    }
    
    /// Handle textDocument/documentSymbol
    pub fn document_symbol(&self, params: DocumentSymbolParams) -> Vec<DocumentSymbol> {
        let docs = self.documents.lock().unwrap();
        
        if let Some(doc) = docs.get(&params.text_document.uri) {
            if let Some(ast) = &doc.ast {
                return self.extract_symbols(ast);
            }
        }
        
        vec![]
    }
    
    /// Handle textDocument/codeAction
    pub fn code_action(&self, params: CodeActionParams) -> Vec<CodeAction> {
        let mut actions = vec![];
        
        for diagnostic in params.context.diagnostics {
            // Generate code actions based on diagnostic
            if diagnostic.message.contains("unused") {
                actions.push(CodeAction {
                    title: "Remove unused variable".to_string(),
                    kind: Some("quickfix".to_string()),
                    edit: None,
                });
            }
        }
        
        actions
    }
    
    /// Parse text and return AST
    fn parse_text(&self, text: &str, filename: &str) -> Program {
        let mut lexer = Lexer::new(text.to_string(), filename.to_string());
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => return Program {
                imports: vec![],
                functions: vec![],
                structs: vec![],
                enums: vec![],
                statements: vec![],
            },
        };
        
        let mut parser = OmniParser::new(tokens, filename.to_string());
        parser.parse().unwrap_or(Program {
            imports: vec![],
            functions: vec![],
            structs: vec![],
            enums: vec![],
            statements: vec![],
        })
    }
    
    /// Convert LSP position to text offset
    fn position_to_offset(&self, text: &str, line: u32, character: u32) -> usize {
        let mut current_line = 0;
        let mut offset = 0;
        
        for c in text.chars() {
            if current_line == line {
                if character as usize <= offset {
                    break;
                }
                offset += 1;
            } else if c == '\n' {
                current_line += 1;
                if current_line > line {
                    break;
                }
            }
        }
        
        offset
    }
    
    /// Extract symbols from AST
    fn extract_symbols(&self, ast: &Program) -> Vec<DocumentSymbol> {
        let mut symbols = vec![];
        
        for func in &ast.functions {
            symbols.push(DocumentSymbol {
                name: func.name.clone(),
                kind: 12, // Function
                range: Range {
                    start: LspPosition { line: 0, character: 0 },
                    end: LspPosition { line: 0, character: 0 },
                },
                children: vec![],
            });
        }
        
        for struct_def in &ast.structs {
            symbols.push(DocumentSymbol {
                name: struct_def.name.clone(),
                kind: 5, // Struct
                range: Range {
                    start: LspPosition { line: 0, character: 0 },
                    end: LspPosition { line: 0, character: 0 },
                },
                children: vec![],
            });
        }
        
        symbols
    }
}

/// Code action
#[derive(Debug, Clone, Serialize)]
pub struct CodeAction {
    pub title: String,
    pub kind: Option<String>,
    pub edit: Option<WorkspaceEdit>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceEdit {
    pub changes: std::collections::HashMap<String, Vec<TextEdit>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// Run the language server
pub fn run_lsp() {
    let server = LanguageServer::new();
    
    // Read JSON-RPC messages from stdin
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    
    // Main loop
    loop {
        // Read message length
        let mut content_length = String::new();
        
        // Read headers
        loop {
            let mut line = String::new();
            if stdin.read_line(&mut line).unwrap() == 0 {
                return;
            }
            
            if line.trim().is_empty() {
                break;
            }
            
            if line.starts_with("Content-Length:") {
                content_length = line.split(':').nth(1).unwrap().trim().to_string();
            }
        }
        
        let length: usize = content_length.parse().unwrap_or(0);
        
        if length == 0 {
            continue;
        }
        
        // Read message body
        let mut body = vec![0u8; length];
        stdin.read_exact(&mut body).unwrap();
        
        // Parse and handle request
        let request: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        // Handle request...
        // For now, just respond to initialize
        let method = request["method"].as_str().unwrap_or("");
        
        match method {
            "initialize" => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request["id"],
                    "result": {
                        "capabilities": server.capabilities
                    }
                });
                
                let response_str = serde_json::to_string(&response).unwrap();
                stdout.write_all(format!("Content-Length: {}\r\n\r\n", response_str.len()).as_bytes()).unwrap();
                stdout.write_all(response_str.as_bytes()).unwrap();
                stdout.flush().unwrap();
            }
            "shutdown" => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request["id"],
                    "result": null
                });
                
                let response_str = serde_json::to_string(&response).unwrap();
                stdout.write_all(format!("Content-Length: {}\r\n\r\n", response_str.len()).as_bytes()).unwrap();
                stdout.write_all(response_str.as_bytes()).unwrap();
                stdout.flush().unwrap();
            }
            "exit" => {
                break;
            }
            _ => {}
        }
    }
}
