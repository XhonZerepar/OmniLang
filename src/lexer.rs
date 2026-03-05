//! Lexer for OmniLang
//! 
//! Tokenizes OmniLang source code into a stream of tokens.
//! Handles whitespace-sensitive syntax, keywords, literals, and operators.
//! Optimized with string interning for identifier performance.

use crate::ast::{BinOp, Literal, Position, Span, UnOp};
use crate::errors::{OmniError, Result};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Global string intern pool for identifiers - reduces memory and speeds up comparison
static IDENT_POOL: LazyLock<HashMap<String, usize>> = LazyLock::new(|| HashMap::new());
static mut IDENT_COUNTER: usize = 0;

/// Token types
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    KwFn,
    KwLet,
    KwMut,
    KwConst,
    KwIf,
    KwElse,
    KwMatch,
    KwLoop,
    KwWhile,
    KwFor,
    KwIn,
    KwReturn,
    KwBreak,
    KwContinue,
    KwStruct,
    KwEnum,
    KwTrait,
    KwImpl,
    KwModule,
    KwImport,
    KwAs,
    KwPub,
    KwUse,
    KwSelf,
    KwSuper,
    KwWhere,
    KwAsync,
    KwAwait,
    KwTry,
    KwCatch,
    KwThrow,
    KwUnsafe,
    KwType,
    KwTrue,
    KwFalse,
    KwNull,
    
    // Types
    TypeInt,
    TypeFloat,
    TypeBool,
    TypeChar,
    TypeString,
    TypeVoid,
    TypeVec,
    TypeHashMap,
    TypeHashSet,
    TypeOption,
    TypeResult,
    
    // Symbols
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Colon,
    DoubleColon,
    Semi,
    At,
    Question,
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Ampersand,
    Pipe,
    Tilde,
    Exclaim,
    
    // Compound operators
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    CaretEq,
    AmpersandEq,
    PipeEq,
    Shl,
    Shr,
    Eq,
    EqEq,
    ExclaimEq,
    Lt,
    LtEq,
   Gt,
    GtEq,
    And,
    AndEq,
    Or,
    OrEq,
    RArrow,
    FatArrow,
    
    // Literals
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Bytes(Vec<u8>),
    
    // Special
    Ident(String),
    Indent,
    Dedent,
    Newline,
    Eof,
    Comment(String),
    
    // Error
    Error(String),
}

impl Token {
    pub fn is_keyword(&self) -> bool {
        matches!(self, 
            Token::KwFn | Token::KwLet | Token::KwMut | Token::KwConst | 
            Token::KwIf | Token::KwElse | Token::KwMatch | Token::KwLoop |
            Token::KwWhile | Token::KwFor | Token::KwIn | Token::KwReturn |
            Token::KwBreak | Token::KwContinue | Token::KwStruct | Token::KwEnum |
            Token::KwTrait | Token::KwImpl | Token::KwModule | Token::KwImport |
            Token::KwAs | Token::KwPub | Token::KwUse | Token::KwSelf |
            Token::KwSuper | Token::KwWhere | Token::KwAsync | Token::KwAwait |
            Token::KwTry | Token::KwCatch | Token::KwThrow | Token::KwUnsafe |
            Token::KwType | Token::KwTrue | Token::KwFalse | Token::KwNull
        )
    }

    pub fn get_keyword(&self) -> Option<String> {
        match self {
            Token::KwFn => Some("fn".to_string()),
            Token::KwLet => Some("let".to_string()),
            Token::KwMut => Some("mut".to_string()),
            Token::KwConst => Some("const".to_string()),
            Token::KwIf => Some("if".to_string()),
            Token::KwElse => Some("else".to_string()),
            Token::KwMatch => Some("match".to_string()),
            Token::KwLoop => Some("loop".to_string()),
            Token::KwWhile => Some("while".to_string()),
            Token::KwFor => Some("for".to_string()),
            Token::KwIn => Some("in".to_string()),
            Token::KwReturn => Some("return".to_string()),
            Token::KwBreak => Some("break".to_string()),
            Token::KwContinue => Some("continue".to_string()),
            Token::KwStruct => Some("struct".to_string()),
            Token::KwEnum => Some("enum".to_string()),
            Token::KwTrait => Some("trait".to_string()),
            Token::KwImpl => Some("impl".to_string()),
            Token::KwModule => Some("module".to_string()),
            Token::KwImport => Some("import".to_string()),
            Token::KwAs => Some("as".to_string()),
            Token::KwPub => Some("pub".to_string()),
            Token::KwUse => Some("use".to_string()),
            Token::KwSelf => Some("self".to_string()),
            Token::KwSuper => Some("super".to_string()),
            Token::KwWhere => Some("where".to_string()),
            Token::KwAsync => Some("async".to_string()),
            Token::KwAwait => Some("await".to_string()),
            Token::KwTry => Some("try".to_string()),
            Token::KwCatch => Some("catch".to_string()),
            Token::KwThrow => Some("throw".to_string()),
            Token::KwUnsafe => Some("unsafe".to_string()),
            Token::KwType => Some("type".to_string()),
            Token::KwTrue => Some("true".to_string()),
            Token::KwFalse => Some("false".to_string()),
            Token::KwNull => Some("null".to_string()),
            _ => None,
        }
    }
}

/// Lexer state machine
pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
    at_line_start: bool,
    filename: String,
}

impl Lexer {
    pub fn new(source: String, filename: String) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0],
            at_line_start: true,
            filename,
        }
    }

    fn current_char(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_char(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(c) = self.current_char() {
            self.pos += 1;
            self.column += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
            }
            Some(c)
        } else {
            None
        }
    }

    fn position(&self) -> Position {
        Position {
            line: self.line,
            column: self.column,
            filename: self.filename.clone(),
        }
    }

    fn span(&self, start: Position) -> Span {
        Span::new(start, self.position())
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        
        while let Some(c) = self.current_char() {
            let start = self.position();
            
            // Handle whitespace (significant indentation)
            if c.is_whitespace() {
                if c == '\n' {
                    tokens.push(Token::Newline);
                    self.advance();
                } else if self.at_line_start {
                    // Track indentation
                    let mut indent = 0;
                    while let Some(c) = self.current_char() {
                        if c == ' ' {
                            indent += 1;
                            self.advance();
                        } else if c == '\t' {
                            indent += 4; // Tab = 4 spaces
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    
                    // Handle indent/dedent
                    let last_indent = *self.indent_stack.last().unwrap_or(&0);
                    if indent > last_indent {
                        self.indent_stack.push(indent);
                        tokens.push(Token::Indent);
                    } else if indent < last_indent {
                        while let Some(&last) = self.indent_stack.last() {
                            if last > indent {
                                self.indent_stack.pop();
                                tokens.push(Token::Dedent);
                            } else {
                                break;
                            }
                        }
                    }
                    self.at_line_start = false;
                } else {
                    self.advance(); // Skip whitespace within line
                }
                continue;
            }
            
            self.at_line_start = false;
            
            // Comments
            if c == '#' {
                let mut comment = String::new();
                while let Some(c) = self.current_char() {
                    if c == '\n' {
                        break;
                    }
                    comment.push(c);
                    self.advance();
                }
                tokens.push(Token::Comment(comment));
                continue;
            }
            
            // Identifiers and keywords
            if c.is_alphabetic() || c == '_' {
                let ident = self.read_ident();
                let token = self.keyword_or_ident(ident);
                tokens.push(token);
                continue;
            }
            
            // Numbers
            if c.is_ascii_digit() {
                let num = self.read_number();
                tokens.push(num);
                continue;
            }
            
            // Strings
            if c == '"' || c == '\'' {
                let string = self.read_string(c)?;
                tokens.push(string);
                continue;
            }
            
            // Multi-character operators
            let token = self.read_operator(c)?;
            if let Some(t) = token {
                tokens.push(t);
            }
        }
        
        // Handle remaining indents at EOF
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(Token::Dedent);
        }
        
        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn read_ident(&mut self) -> String {
        let mut ident = String::new();
        while let Some(c) = self.current_char() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }

    fn keyword_or_ident(&self, ident: String) -> Token {
        match ident.as_str() {
            // Keywords
            "fn" => Token::KwFn,
            "let" => Token::KwLet,
            "mut" => Token::KwMut,
            "const" => Token::KwConst,
            "if" => Token::KwIf,
            "else" => Token::KwElse,
            "match" => Token::KwMatch,
            "loop" => Token::KwLoop,
            "while" => Token::KwWhile,
            "for" => Token::KwFor,
            "in" => Token::KwIn,
            "return" => Token::KwReturn,
            "break" => Token::KwBreak,
            "continue" => Token::KwContinue,
            "struct" => Token::KwStruct,
            "enum" => Token::KwEnum,
            "trait" => Token::KwTrait,
            "impl" => Token::KwImpl,
            "module" => Token::KwModule,
            "import" => Token::KwImport,
            "as" => Token::KwAs,
            "pub" => Token::KwPub,
            "use" => Token::KwUse,
            "self" => Token::KwSelf,
            "super" => Token::KwSuper,
            "where" => Token::KwWhere,
            "async" => Token::KwAsync,
            "await" => Token::KwAwait,
            "try" => Token::KwTry,
            "catch" => Token::KwCatch,
            "throw" => Token::KwThrow,
            "unsafe" => Token::KwUnsafe,
            "type" => Token::KwType,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "null" => Token::KwNull,
            
            // Type keywords
            "Int" => Token::TypeInt,
            "Float" => Token::TypeFloat,
            "Bool" => Token::TypeBool,
            "Char" => Token::TypeChar,
            "String" => Token::TypeString,
            "Void" => Token::TypeVoid,
            "Vec" => Token::TypeVec,
            "HashMap" => Token::TypeHashMap,
            "HashSet" => Token::TypeHashSet,
            "Option" => Token::TypeOption,
            "Result" => Token::TypeResult,
            
            _ => Token::Ident(ident),
        }
    }

    fn read_number(&mut self) -> Token {
        let mut num_str = String::new();
        let mut is_float = false;
        let start = self.position();
        
        while let Some(c) = self.current_char() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                if let Some(next) = self.peek_char() {
                    if next.is_ascii_digit() {
                        is_float = true;
                        num_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else if c == 'e' || c == 'E' {
                is_float = true;
                num_str.push(c);
                self.advance();
                if let Some(sign) = self.current_char() {
                    if sign == '+' || sign == '-' {
                        num_str.push(sign);
                        self.advance();
                    }
                }
            } else {
                break;
            }
        }
        
        if is_float {
            match num_str.parse::<f64>() {
                Ok(n) => Token::Float(n),
                Err(_) => Token::Error(format!("Invalid float: {}", num_str)),
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(n) => Token::Int(n),
                Err(_) => Token::Error(format!("Invalid integer: {}", num_str)),
            }
        }
    }

    fn read_string(&mut self, quote: char) -> Result<Token> {
        let start = self.position();
        self.advance(); // Skip opening quote
        
        let mut string = String::new();
        let mut is_bytes = false;
        
        while let Some(c) = self.current_char() {
            if c == quote {
                self.advance();
                // Check for raw string prefix
                if string.starts_with('r') {
                    is_bytes = true;
                    string = string[1..].to_string();
                }
                return if is_bytes {
                    Ok(Token::Bytes(string.into_bytes()))
                } else {
                    Ok(Token::String(string))
                };
            } else if c == '\\' {
                self.advance();
                let escaped = match self.current_char() {
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('r') => '\r',
                    Some('\\') => '\\',
                    Some('\'') => '\'',
                    Some('"') => '"',
                    Some('0') => '\0',
                    Some('x') => {
                        // Hex escape
                        let hex: String = (0..2).filter_map(|_| {
                            self.current_char().and_then(|c| {
                                self.advance();
                                if c.is_ascii_hexdigit() {
                                    Some(c)
                                } else {
                                    None
                                }
                            })
                        }).collect();
                        match u8::from_str_radix(&hex, 16) {
                            Ok(b) => b as char,
                            Err(_) => return Err(OmniError::LexerError {
                                location: start.to_string(),
                                message: format!("Invalid hex escape: {}", hex),
                            }),
                        }
                    }
                    Some(c) => c,
                    None => return Err(OmniError::LexerError {
                        location: start.to_string(),
                        message: "Unterminated string".to_string(),
                    }),
                };
                string.push(escaped);
            } else if c == '\n' && quote == '\'' {
                return Err(OmniError::LexerError {
                    location: start.to_string(),
                    message: "Newline in character literal".to_string(),
                });
            } else {
                string.push(c);
                self.advance();
            }
        }
        
        Err(OmniError::LexerError {
            location: start.to_string(),
            message: "Unterminated string".to_string(),
        })
    }

    fn read_operator(&mut self, c: char) -> Result<Option<Token>> {
        let token = match c {
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            ',' => Token::Comma,
            '.' => {
                // Check for range operator (.. or ..=)
                if let Some(next) = self.peek_char() {
                    if next == '.' {
                        self.advance();
                        if let Some(after) = self.peek_char() {
                            if after == '=' {
                                self.advance();
                                return Ok(Some(Token::Range));
                            }
                        }
                        return Ok(Some(Token::Range));
                    }
                }
                Token::Dot
            }
            ':' => {
                if let Some(next) = self.peek_char() {
                    if next == ':' {
                        self.advance();
                        return Ok(Some(Token::DoubleColon));
                    }
                }
                Token::Colon
            }
            ';' => Token::Semi,
            '@' => Token::At,
            '?' => Token::Question,
            
            '+' => {
                if let Some(next) = self.peek_char() {
                    if next == '=' {
                        self.advance();
                        return Ok(Some(Token::PlusEq));
                    }
                }
                Token::Plus
            }
            '-' => {
                if let Some(next) = self.peek_char() {
                    if next == '=' {
                        self.advance();
                        return Ok(Some(Token::MinusEq));
                    } else if next == '>' {
                        self.advance();
                        return Ok(Some(Token::RArrow));
                    }
                }
                Token::Minus
            }
            '*' => {
                if let Some(next) = self.peek_char() {
                    if next == '=' {
                        self.advance();
                        return Ok(Some(Token::StarEq));
                    }
                }
                Token::Star
            }
            '/' => {
                if let Some(next) = self.peek_char() {
                    if next == '=' {
                        self.advance();
                        return Ok(Some(Token::SlashEq));
                    }
                }
                Token::Slash
            }
            '%' => {
                if let Some(next) = self.peek_char() {
                    if next == '=' {
                        self.advance();
                        return Ok(Some(Token::PercentEq));
                    }
                }
                Token::Percent
            }
            '^' => {
                if let Some(next) = self.peek_char() {
                    if next == '=' {
                        self.advance();
                        return Ok(Some(Token::CaretEq));
                    }
                }
                Token::Caret
            }
            '&' => {
                if let Some(next) = self.peek_char() {
                    if next == '=' {
                        self.advance();
                        return Ok(Some(Token::AmpersandEq));
                    } else if next == '&' {
                        self.advance();
                        return Ok(Some(Token::And));
                    }
                }
                Token::Ampersand
            }
            '|' => {
                if let Some(next) = self.peek_char() {
                    if next == '=' {
                        self.advance();
                        return Ok(Some(Token::PipeEq));
                    } else if next == '|' {
                        self.advance();
                        return Ok(Some(Token::Or));
                    }
                }
                Token::Pipe
            }
            '!' => {
                if let Some(next) = self.peek_char() {
                    if next == '=' {
                        self.advance();
                        return Ok(Some(Token::ExclaimEq));
                    }
                }
                Token::Exclaim
            }
            '=' => {
                if let Some(next) = self.peek_char() {
                    if next == '=' {
                        self.advance();
                        return Ok(Some(Token::EqEq));
                    } else if next == '>' {
                        self.advance();
                        return Ok(Some(Token::FatArrow));
                    }
                }
                Token::Eq
            }
            '<' => {
                if let Some(next) = self.peek_char() {
                    if next == '=' {
                        self.advance();
                        return Ok(Some(Token::LtEq));
                    } else if next == '<' {
                        self.advance();
                        return Ok(Some(Token::Shl));
                    }
                }
                Token::Lt
            }
            '>' => {
                if let Some(next) = self.peek_char() {
                    if next == '=' {
                        self.advance();
                        return Ok(Some(Token::GtEq));
                    } else if next == '>' {
                        self.advance();
                        return Ok(Some(Token::Shr));
                    }
                }
                Token::Gt
            }
            '~' => Token::Tilde,
            
            _ => {
                return Err(OmniError::LexerError {
                    location: self.position().to_string(),
                    message: format!("Unknown character: {}", c),
                });
            }
        };
        
        self.advance();
        Ok(Some(token))
    }
}

/// Convert tokens back to source code (for debugging)
pub fn tokens_to_string(tokens: &[Token]) -> String {
    tokens.iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>().join(", ")
}
