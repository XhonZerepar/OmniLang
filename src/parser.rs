//! Parser for OmniLang
//! 
//! Transforms the token stream into an Abstract Syntax Tree (AST).
//! Uses recursive descent parsing with operator precedence.

use crate::ast::*;
use crate::errors::{OmniError, Result};
use crate::lexer::{Token, Lexer};

/// Parser context for tracking state during parsing
pub struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    filename: String,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, filename: String) -> Self {
        Self {
            tokens,
            pos: 0,
            filename,
            _phantom: std::marker::PhantomData,
        }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn peek(&self, offset: usize) -> &Token {
        self.tokens.get(self.pos + offset).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, expected: Token) -> Result<Token> {
        let current = self.current().clone();
        if std::mem::discriminant(&current) == std::mem::discriminant(&expected) {
            Ok(self.advance())
        } else {
            Err(OmniError::ParserError {
                location: self.location(),
                message: format!("Expected {:?}, got {:?}", expected, current),
            })
        }
    }

    fn location(&self) -> String {
        format!("{}:{}", self.filename, self.pos)
    }

    fn span(&self, start: Position) -> Span {
        Span {
            start,
            end: Position {
                line: 0,
                column: 0,
                filename: self.filename.clone(),
            },
        }
    }

    fn position(&self) -> Position {
        Position {
            line: 0,
            column: 0,
            filename: self.filename.clone(),
        }
    }

    /// Parse the token stream into a Program
    pub fn parse(&mut self) -> Result<Program> {
        let mut imports = Vec::new();
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut statements = Vec::new();

        while !matches!(self.current(), Token::Eof) {
            match self.current() {
                Token::KwImport => {
                    imports.push(self.parse_import()?);
                }
                Token::KwFn => {
                    functions.push(self.parse_function()?);
                }
                Token::KwStruct => {
                    structs.push(self.parse_struct()?);
                }
                Token::KwEnum => {
                    enums.push(self.parse_enum()?);
                }
                Token::Newline | Token::Indent | Token::Dedent => {
                    self.advance(); // Skip formatting tokens
                }
                _ => {
                    statements.push(self.parse_statement()?);
                }
            }
        }

        Ok(Program {
            imports,
            functions,
            structs,
            enums,
            statements,
        })
    }

    fn parse_import(&mut self) -> Result<Import> {
        let start = self.position();
        self.expect(Token::KwImport)?;
        
        let path = match self.current() {
            Token::String(s) => {
                let p = s.clone();
                self.advance();
                p
            }
            Token::Ident(id) => {
                let mut path = id.clone();
                while matches!(self.peek(0), Token::Dot) {
                    self.advance(); // consume dot
                    if let Token::Ident(id) = self.advance() {
                        path.push('.');
                        path.push_str(&id);
                    }
                }
                path
            }
            _ => {
                return Err(OmniError::ParserError {
                    location: self.location(),
                    message: "Expected import path".to_string(),
                });
            }
        };

        let alias = if matches!(self.current(), Token::KwAs) {
            self.advance();
            if let Token::Ident(id) = self.advance() {
                Some(id)
            } else {
                return Err(OmniError::ParserError {
                    location: self.location(),
                    message: "Expected identifier after 'as'".to_string(),
                });
            }
        } else {
            None
        };

        let items = if matches!(self.current(), Token::LBrace) {
            self.advance();
            let mut items = Vec::new();
            while !matches!(self.current(), Token::RBrace) {
                if let Token::Ident(id) = self.advance() {
                    items.push(id);
                }
                if matches!(self.current(), Token::Comma) {
                    self.advance();
                }
            }
            self.advance(); // consume RBrace
            items
        } else {
            Vec::new()
        };

        Ok(Import {
            path,
            alias,
            items,
            span: self.span(start),
        })
    }

    fn parse_function(&mut self) -> Result<Function> {
        let start = self.position();
        self.expect(Token::KwFn)?;
        
        let name = match self.advance() {
            Token::Ident(id) => id,
            _ => {
                return Err(OmniError::ParserError {
                    location: self.location(),
                    message: "Expected function name".to_string(),
                });
            }
        };

        // Parse generic parameters (TODO)
        
        // Parse parameters
        self.expect(Token::LParen)?;
        let mut args = Vec::new();
        while !matches!(self.current(), Token::RParen) {
            let arg_name = match self.advance() {
                Token::Ident(id) => id,
                _ => {
                    return Err(OmniError::ParserError {
                        location: self.location(),
                        message: "Expected argument name".to_string(),
                    });
                }
            };
            
            self.expect(Token::Colon)?;
            let arg_type = self.parse_type()?;
            
            let default = if matches!(self.current(), Token::Eq) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            args.push(Arg {
                name: arg_name,
                arg_type,
                default,
                span: self.span(start.clone()),
            });
            
            if matches!(self.current(), Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RParen)?;

        // Parse return type
        let return_type = if matches!(self.current(), Token::RArrow) {
            self.advance();
            self.parse_type()?
        } else {
            Type::Void
        };

        // Parse function body
        let body = if matches!(self.current(), Token::LBrace) {
            self.advance();
            self.parse_block(Token::RBrace)?
        } else if matches!(self.current(), Token::Colon) {
            // Indented body
            self.advance();
            self.parse_block(Token::Dedent)?
        } else {
            Vec::new()
        };

        Ok(Function {
            name,
            args,
            return_type,
            body,
            span: self.span(start),
            is_public: false,
            is_async: false,
            attrs: Vec::new(),
        })
    }

    fn parse_struct(&mut self) -> Result<StructDef> {
        let start = self.position();
        self.expect(Token::KwStruct)?;
        
        let name = match self.advance() {
            Token::Ident(id) => id,
            _ => {
                return Err(OmniError::ParserError {
                    location: self.location(),
                    message: "Expected struct name".to_string(),
                });
            }
        };

        // Parse fields
        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();
        while !matches!(self.current(), Token::RBrace) {
            let field_name = match self.advance() {
                Token::Ident(id) => id,
                _ => {
                    return Err(OmniError::ParserError {
                        location: self.location(),
                        message: "Expected field name".to_string(),
                    });
                }
            };
            
            self.expect(Token::Colon)?;
            let field_type = self.parse_type()?;
            
            fields.push(Field {
                name: field_name,
                field_type,
                span: self.span(start.clone()),
            });
            
            if matches!(self.current(), Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RBrace)?;

        Ok(StructDef {
            name,
            fields,
            methods: Vec::new(),
            span: self.span(start),
            is_public: false,
        })
    }

    fn parse_enum(&mut self) -> Result<EnumDef> {
        let start = self.position();
        self.expect(Token::KwEnum)?;
        
        let name = match self.advance() {
            Token::Ident(id) => id,
            _ => {
                return Err(OmniError::ParserError {
                    location: self.location(),
                    message: "Expected enum name".to_string(),
                });
            }
        };

        // Parse variants
        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();
        while !matches!(self.current(), Token::RBrace) {
            let variant_name = match self.advance() {
                Token::Ident(id) => id,
                _ => {
                    return Err(OmniError::ParserError {
                        location: self.location(),
                        message: "Expected variant name".to_string(),
                    });
                }
            };
            
            // Parse associated data
            let mut data = Vec::new();
            if matches!(self.current(), Token::LParen) {
                self.advance();
                while !matches!(self.current(), Token::RParen) {
                    data.push(self.parse_type()?);
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RParen)?;
            }
            
            variants.push(EnumVariant {
                name: variant_name,
                data,
                span: self.span(start.clone()),
            });
            
            if matches!(self.current(), Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RBrace)?;

        Ok(EnumDef {
            name,
            variants,
            span: self.span(start),
            is_public: false,
        })
    }

    fn parse_type(&mut self) -> Result<Type> {
        let t = match self.current().clone() {
            Token::TypeInt => {
                self.advance();
                Type::Int
            }
            Token::TypeFloat => {
                self.advance();
                Type::Float
            }
            Token::TypeBool => {
                self.advance();
                Type::Bool
            }
            Token::TypeChar => {
                self.advance();
                Type::Char
            }
            Token::TypeString => {
                self.advance();
                Type::String
            }
            Token::TypeVoid => {
                self.advance();
                Type::Void
            }
            Token::TypeVec => {
                self.advance();
                self.expect(Token::Lt)?;
                let inner = self.parse_type()?;
                self.expect(Token::Gt)?;
                Type::Vec(Box::new(inner))
            }
            Token::TypeHashMap => {
                self.advance();
                self.expect(Token::Lt)?;
                let key = self.parse_type()?;
                self.expect(Token::Comma)?;
                let value = self.parse_type()?;
                self.expect(Token::Gt)?;
                Type::HashMap(Box::new(key), Box::new(value))
            }
            Token::TypeHashSet => {
                self.advance();
                self.expect(Token::Lt)?;
                let inner = self.parse_type()?;
                self.expect(Token::Gt)?;
                Type::HashSet(Box::new(inner))
            }
            Token::TypeOption => {
                self.advance();
                self.expect(Token::Lt)?;
                let inner = self.parse_type()?;
                self.expect(Token::Gt)?;
                Type::Option(Box::new(inner))
            }
            Token::TypeResult => {
                self.advance();
                self.expect(Token::Lt)?;
                let ok = self.parse_type()?;
                self.expect(Token::Comma)?;
                let err = self.parse_type()?;
                self.expect(Token::Gt)?;
                Type::Result(Box::new(ok), Box::new(err))
            }
            Token::Ident(id) => {
                self.advance();
                // Check for generic parameters
                if matches!(self.current(), Token::Lt) {
                    self.advance();
                    let mut params = Vec::new();
                    while !matches!(self.current(), Token::Gt) {
                        params.push(self.parse_type()?);
                        if matches!(self.current(), Token::Comma) {
                            self.advance();
                        }
                    }
                    self.expect(Token::Gt)?;
                    // For now, just return custom type
                    Type::Custom(id)
                } else {
                    Type::Custom(id)
                }
            }
            Token::Star => {
                self.advance();
                let inner = self.parse_type()?;
                Type::Ptr(Box::new(inner))
            }
            Token::Ampersand => {
                self.advance();
                let mutable = matches!(self.current(), Token::KwMut);
                if mutable {
                    self.advance();
                }
                let inner = self.parse_type()?;
                if mutable {
                    Type::MutRef(Box::new(inner))
                } else {
                    Type::Ref(Box::new(inner))
                }
            }
            Token::LBracket => {
                self.advance();
                let inner = self.parse_type()?;
                let size = if matches!(self.current(), Token::Semi) {
                    self.advance();
                    if let Token::Int(n) = self.advance() {
                        Some(n as usize)
                    } else {
                        None
                    }
                } else {
                    None
                };
                self.expect(Token::RBracket)?;
                Type::Array(Box::new(inner), size)
            }
            Token::LParen => {
                // Function type
                self.advance();
                let mut args = Vec::new();
                while !matches!(self.current(), Token::RParen) {
                    args.push(self.parse_type()?);
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RParen)?;
                self.expect(Token::RArrow)?;
                let ret = self.parse_type()?;
                Type::Function(args, Box::new(ret))
            }
            _ => {
                return Err(OmniError::ParserError {
                    location: self.location(),
                    message: format!("Expected type, got {:?}", self.current()),
                });
            }
        };
        
        Ok(t)
    }

    fn parse_block(&mut self, end_token: Token) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        
        while !matches!(self.current(), end_token) && !matches!(self.current(), Token::Eof) {
            // Skip newlines and indentation changes
            if matches!(self.current(), Token::Newline | Token::Indent | Token::Dedent) {
                self.advance();
                continue;
            }
            
            statements.push(self.parse_statement()?);
        }
        
        if !matches!(self.current(), Token::Eof) {
            self.advance(); // consume end token
        }
        
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Stmt> {
        let start = self.position();
        
        match self.current().clone() {
            Token::KwLet => self.parse_let(false),
            Token::KwMut => self.parse_mut(),
            Token::KwReturn => self.parse_return(),
            Token::KwIf => self.parse_if(),
            Token::KwMatch => self.parse_match(),
            Token::KwLoop => self.parse_loop(),
            Token::KwWhile => self.parse_while(),
            Token::KwFor => self.parse_for(),
            Token::KwBreak => {
                self.advance();
                Ok(Stmt::Break { span: self.span(start) })
            }
            Token::KwContinue => {
                self.advance();
                Ok(Stmt::Continue { span: self.span(start) })
            }
            Token::KwUnsafe => self.parse_unsafe(),
            Token::KwTry => self.parse_try(),
            Token::KwThrow => self.parse_throw(),
            Token::LBrace => {
                self.advance();
                let stmts = self.parse_block(Token::RBrace)?;
                Ok(Stmt::Block {
                    statements: stmts,
                    span: self.span(start),
                })
            }
            Token::Newline => {
                self.advance();
                self.parse_statement()
            }
            _ => {
                let expr = self.parse_expression()?;
                Ok(Stmt::ExprStmt(expr))
            }
        }
    }

    fn parse_let(&mut self, mutable: bool) -> Result<Stmt> {
        let start = self.position();
        self.advance(); // consume let/mut
        
        let name = match self.advance() {
            Token::Ident(id) => id,
            _ => {
                return Err(OmniError::ParserError {
                    location: self.location(),
                    message: "Expected variable name".to_string(),
                });
            }
        };

        let var_type = if matches!(self.current(), Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(Token::Eq)?;
        let value = self.parse_expression()?;

        if mutable {
            Ok(Stmt::Mut {
                name,
                var_type,
                value,
                span: self.span(start),
            })
        } else {
            Ok(Stmt::Let {
                name,
                var_type,
                value,
                span: self.span(start),
            })
        }
    }

    fn parse_mut(&mut self) -> Result<Stmt> {
        self.parse_let(true)
    }

    fn parse_return(&mut self) -> Result<Stmt> {
        let start = self.position();
        self.advance();
        
        if matches!(self.current(), Token::Newline) || matches!(self.current(), Token::Eof) {
            Ok(Stmt::Return {
                value: None,
                span: self.span(start),
            })
        } else {
            let value = self.parse_expression()?;
            Ok(Stmt::Return {
                value: Some(value),
                span: self.span(start),
            })
        }
    }

    fn parse_if(&mut self) -> Result<Stmt> {
        let start = self.position();
        self.advance(); // consume if
        
        let condition = self.parse_expression()?;
        
        let then_branch = if matches!(self.current(), Token::LBrace) {
            self.advance();
            self.parse_block(Token::RBrace)?
        } else if matches!(self.current(), Token::Colon) {
            self.advance();
            self.parse_block(Token::Dedent)?
        } else {
            vec![self.parse_statement()?]
        };
        
        let else_branch = if matches!(self.current(), Token::KwElse) {
            self.advance();
            if matches!(self.current(), Token::KwIf) {
                Some(vec![self.parse_if()?])
            } else if matches!(self.current(), Token::LBrace) {
                self.advance();
                Some(self.parse_block(Token::RBrace)?)
            } else if matches!(self.current(), Token::Colon) {
                self.advance();
                Some(self.parse_block(Token::Dedent)?)
            } else {
                None
            }
        } else {
            None
        };
        
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
            span: self.span(start),
        })
    }

    fn parse_match(&mut self) -> Result<Stmt> {
        let start = self.position();
        self.advance(); // consume match
        
        let expr = self.parse_expression()?;
        
        self.expect(Token::LBrace)?;
        
        let mut arms = Vec::new();
        while !matches!(self.current(), Token::RBrace) {
            let pattern = self.parse_pattern()?;
            
            let guard = if matches!(self.current(), Token::KwIf) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            self.expect(Token::FatArrow)?;
            
            let body = if matches!(self.current(), Token::LBrace) {
                self.advance();
                self.parse_block(Token::RBrace)?
            } else {
                vec![self.parse_statement()?]
            };
            
            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span: self.span(start.clone()),
            });
            
            if matches!(self.current(), Token::Comma) {
                self.advance();
            }
        }
        
        self.expect(Token::RBrace)?;
        
        Ok(Stmt::Match {
            expr,
            arms,
            span: self.span(start),
        })
    }

    fn parse_pattern(&mut self) -> Result<Pat> {
        let pat = match self.current().clone() {
            Token::Underscore => {
                self.advance();
                Pat::Wildcard
            }
            Token::Int(n) => {
                self.advance();
                Pat::Lit(Literal::Int(n))
            }
            Token::Float(n) => {
                self.advance();
                Pat::Lit(Literal::Float(n))
            }
            Token::Bool(b) => {
                self.advance();
                Pat::Lit(Literal::Bool(b))
            }
            Token::String(s) => {
                self.advance();
                Pat::Lit(Literal::String(s))
            }
            Token::Char(c) => {
                self.advance();
                Pat::Lit(Literal::Char(c))
            }
            Token::Ident(id) => {
                self.advance();
                // Check for enum variant pattern
                if matches!(self.current(), Token::DoubleColon) {
                    self.advance(); // consume ::
                    let variant = match self.advance() {
                        Token::Ident(v) => v,
                        _ => {
                            return Err(OmniError::ParserError {
                                location: self.location(),
                                message: "Expected variant name".to_string(),
                            });
                        }
                    };
                    
                    let mut data = Vec::new();
                    if matches!(self.current(), Token::LParen) {
                        self.advance();
                        while !matches!(self.current(), Token::RParen) {
                            data.push(self.parse_pattern()?);
                            if matches!(self.current(), Token::Comma) {
                                self.advance();
                            }
                        }
                        self.expect(Token::RParen)?;
                    }
                    
                    Pat::Enum {
                        name: id,
                        variant,
                        data,
                    }
                } else {
                    Pat::Ident(id)
                }
            }
            Token::LParen => {
                self.advance();
                let mut patterns = Vec::new();
                while !matches!(self.current(), Token::RParen) {
                    patterns.push(self.parse_pattern()?);
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RParen)?;
                Pat::Tuple(patterns)
            }
            _ => {
                return Err(OmniError::ParserError {
                    location: self.location(),
                    message: format!("Expected pattern, got {:?}", self.current()),
                });
            }
        };
        
        Ok(pat)
    }

    fn parse_loop(&mut self) -> Result<Stmt> {
        let start = self.position();
        self.advance(); // consume loop
        
        let body = if matches!(self.current(), Token::LBrace) {
            self.advance();
            self.parse_block(Token::RBrace)?
        } else if matches!(self.current(), Token::Colon) {
            self.advance();
            self.parse_block(Token::Dedent)?
        } else {
            vec![self.parse_statement()?]
        };
        
        Ok(Stmt::Loop {
            body,
            span: self.span(start),
        })
    }

    fn parse_while(&mut self) -> Result<Stmt> {
        let start = self.position();
        self.advance(); // consume while
        
        let condition = self.parse_expression()?;
        
        let body = if matches!(self.current(), Token::LBrace) {
            self.advance();
            self.parse_block(Token::RBrace)?
        } else if matches!(self.current(), Token::Colon) {
            self.advance();
            self.parse_block(Token::Dedent)?
        } else {
            vec![self.parse_statement()?]
        };
        
        Ok(Stmt::While {
            condition,
            body,
            span: self.span(start),
        })
    }

    fn parse_for(&mut self) -> Result<Stmt> {
        let start = self.position();
        self.advance(); // consume for
        
        let variable = match self.advance() {
            Token::Ident(id) => id,
            _ => {
                return Err(OmniError::ParserError {
                    location: self.location(),
                    message: "Expected loop variable".to_string(),
                });
            }
        };
        
        self.expect(Token::KwIn)?;
        let iter = self.parse_expression()?;
        
        let body = if matches!(self.current(), Token::LBrace) {
            self.advance();
            self.parse_block(Token::RBrace)?
        } else if matches!(self.current(), Token::Colon) {
            self.advance();
            self.parse_block(Token::Dedent)?
        } else {
            vec![self.parse_statement()?]
        };
        
        Ok(Stmt::For {
            variable,
            iter,
            body,
            span: self.span(start),
        })
    }

    fn parse_unsafe(&mut self) -> Result<Stmt> {
        let start = self.position();
        self.advance();
        
        let body = if matches!(self.current(), Token::LBrace) {
            self.advance();
            self.parse_block(Token::RBrace)?
        } else {
            vec![self.parse_statement()?]
        };
        
        Ok(Stmt::Unsafe {
            body,
            span: self.span(start),
        })
    }

    fn parse_try(&mut self) -> Result<Stmt> {
        let start = self.position();
        self.advance();
        
        let body = if matches!(self.current(), Token::LBrace) {
            self.advance();
            self.parse_block(Token::RBrace)?
        } else {
            vec![self.parse_statement()?]
        };
        
        let mut catches = Vec::new();
        while matches!(self.current(), Token::KwCatch) {
            self.advance();
            
            let exception_type = if matches!(self.current(), Token::LParen) {
                self.advance();
                let t = self.parse_type()?;
                self.expect(Token::RParen)?;
                Some(t)
            } else {
                None
            };
            
            let variable = if matches!(self.current(), Token::Ident(_)) {
                if let Token::Ident(id) = self.advance() {
                    Some(id)
                } else {
                    None
                }
            } else {
                None
            };
            
            let catch_body = if matches!(self.current(), Token::LBrace) {
                self.advance();
                self.parse_block(Token::RBrace)?
            } else {
                vec![self.parse_statement()?]
            };
            
            catches.push(CatchClause {
                exception_type,
                variable,
                body: catch_body,
                span: self.span(start.clone()),
            });
        }
        
        let finally = if matches!(self.current(), Token::KwFinally) {
            self.advance();
            if matches!(self.current(), Token::LBrace) {
                self.advance();
                Some(self.parse_block(Token::RBrace)?)
            } else {
                Some(vec![self.parse_statement()?])
            }
        } else {
            None
        };
        
        Ok(Stmt::Try {
            body,
            catches,
            finally,
            span: self.span(start),
        })
    }

    fn parse_throw(&mut self) -> Result<Stmt> {
        let start = self.position();
        self.advance();
        
        let expr = self.parse_expression()?;
        
        Ok(Stmt::Throw {
            expr,
            span: self.span(start),
        })
    }

    // Expression parsing with precedence
    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_assign_expr()
    }

    fn parse_assign_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_or_expr()?;
        
        if matches!(self.current(), Token::Eq) {
            self.advance();
            let right = self.parse_assign_expr()?;
            return Ok(Expr::Assign {
                target: Box::new(left),
                value: Box::new(right),
                span: Span::dummy(),
            });
        }
        
        Ok(left)
    }

    fn parse_or_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_and_expr()?;
        
        while matches!(self.current(), Token::Or) {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
                span: Span::dummy(),
            };
        }
        
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_bitwise_or_expr()?;
        
        while matches!(self.current(), Token::And) {
            self.advance();
            let right = self.parse_bitwise_or_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
                span: Span::dummy(),
            };
        }
        
        Ok(left)
    }

    fn parse_bitwise_or_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_bitwise_xor_expr()?;
        
        while matches!(self.current(), Token::Pipe) {
            self.advance();
            let right = self.parse_bitwise_xor_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::BitOr,
                right: Box::new(right),
                span: Span::dummy(),
            };
        }
        
        Ok(left)
    }

    fn parse_bitwise_xor_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_bitwise_and_expr()?;
        
        while matches!(self.current(), Token::Caret) {
            self.advance();
            let right = self.parse_bitwise_and_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::BitXor,
                right: Box::new(right),
                span: Span::dummy(),
            };
        }
        
        Ok(left)
    }

    fn parse_bitwise_and_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality_expr()?;
        
        while matches!(self.current(), Token::Ampersand) {
            self.advance();
            let right = self.parse_equality_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::BitAnd,
                right: Box::new(right),
                span: Span::dummy(),
            };
        }
        
        Ok(left)
    }

    fn parse_equality_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_relational_expr()?;
        
        while matches!(self.current(), Token::EqEq) || matches!(self.current(), Token::ExclaimEq) {
            let op = match self.advance() {
                Token::EqEq => BinOp::Eq,
                Token::ExclaimEq => BinOp::Ne,
                _ => unreachable!(),
            };
            let right = self.parse_relational_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: Span::dummy(),
            };
        }
        
        Ok(left)
    }

    fn parse_relational_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_shift_expr()?;
        
        while matches!(self.current(), Token::Lt) || matches!(self.current(), Token::LtEq) ||
               matches!(self.current(), Token::Gt) || matches!(self.current(), Token::GtEq) {
            let op = match self.advance() {
                Token::Lt => BinOp::Lt,
                Token::LtEq => BinOp::Le,
                Token::Gt => BinOp::Gt,
                Token::GtEq => BinOp::Ge,
                _ => unreachable!(),
            };
            let right = self.parse_shift_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: Span::dummy(),
            };
        }
        
        Ok(left)
    }

    fn parse_shift_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_additive_expr()?;
        
        while matches!(self.current(), Token::Shl) || matches!(self.current(), Token::Shr) {
            let op = match self.advance() {
                Token::Shl => BinOp::Shl,
                Token::Shr => BinOp::Shr,
                _ => unreachable!(),
            };
            let right = self.parse_additive_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: Span::dummy(),
            };
        }
        
        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplicative_expr()?;
        
        while matches!(self.current(), Token::Plus) || matches!(self.current(), Token::Minus) {
            let op = match self.advance() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_multiplicative_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: Span::dummy(),
            };
        }
        
        Ok(left)
    }

    fn parse_multiplicative_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary_expr()?;
        
        while matches!(self.current(), Token::Star) || matches!(self.current(), Token::Slash) ||
               matches!(self.current(), Token::Percent) {
            let op = match self.advance() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => unreachable!(),
            };
            let right = self.parse_unary_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: Span::dummy(),
            };
        }
        
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr> {
        let start = self.position();
        
        match self.current().clone() {
            Token::Exclaim => {
                self.advance();
                let operand = Box::new(self.parse_unary_expr()?);
                Ok(Expr::Unary {
                    op: UnOp::Not,
                    operand,
                    span: self.span(start),
                })
            }
            Token::Minus => {
                self.advance();
                let operand = Box::new(self.parse_unary_expr()?);
                Ok(Expr::Unary {
                    op: UnOp::Neg,
                    operand,
                    span: self.span(start),
                })
            }
            Token::Tilde => {
                self.advance();
                let operand = Box::new(self.parse_unary_expr()?);
                Ok(Expr::Unary {
                    op: UnOp::BitNot,
                    operand,
                    span: self.span(start),
                })
            }
            Token::Ampersand => {
                self.advance();
                let mutable = matches!(self.current(), Token::KwMut);
                if mutable {
                    self.advance();
                }
                let expr = Box::new(self.parse_unary_expr()?);
                Ok(Expr::Borrow {
                    mutable,
                    expr,
                    span: self.span(start),
                })
            }
            Token::Star => {
                self.advance();
                let expr = Box::new(self.parse_unary_expr()?);
                Ok(Expr::Unary {
                    op: UnOp::Deref,
                    operand: expr,
                    span: self.span(start),
                })
            }
            Token::KwAsync => {
                self.advance();
                let body = if matches!(self.current(), Token::LBrace) {
                    self.advance();
                    self.parse_block(Token::RBrace)?
                } else {
                    vec![self.parse_statement()?]
                };
                Ok(Stmt::Async {
                    body,
                    span: self.span(start),
                }.into())
            }
            Token::KwAwait => {
                self.advance();
                let expr = Box::new(self.parse_unary_expr()?);
                Ok(Expr::Await {
                    expr,
                    span: self.span(start),
                })
            }
            _ => self.parse_postfix_expr(),
        }
    }

    fn parse_postfix_expr(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary_expr()?;
        
        loop {
            let start = self.position();
            
            match self.current().clone() {
                Token::Dot => {
                    self.advance();
                    let field = match self.advance() {
                        Token::Ident(id) => id,
                        _ => {
                            return Err(OmniError::ParserError {
                                location: self.location(),
                                message: "Expected field name".to_string(),
                            });
                        }
                    };
                    
                    // Check for method call
                    if matches!(self.current(), Token::LParen) {
                        self.advance();
                        let mut args = Vec::new();
                        while !matches!(self.current(), Token::RParen) {
                            args.push(self.parse_expression()?);
                            if matches!(self.current(), Token::Comma) {
                                self.advance();
                            }
                        }
                        self.expect(Token::RParen)?;
                        expr = Expr::MethodCall {
                            object: Box::new(expr),
                            method: field,
                            args,
                            span: self.span(start),
                        };
                    } else {
                        expr = Expr::FieldAccess {
                            object: Box::new(expr),
                            field,
                            span: self.span(start),
                        };
                    }
                }
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(Token::RBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                        span: self.span(start),
                    };
                }
                Token::LParen => {
                    // Function call
                    self.advance();
                    let mut args = Vec::new();
                    while !matches!(self.current(), Token::RParen) {
                        args.push(self.parse_expression()?);
                        if matches!(self.current(), Token::Comma) {
                            self.advance();
                        }
                    }
                    self.expect(Token::RParen)?;
                    expr = Expr::Call {
                        func: Box::new(expr),
                        args,
                        span: self.span(start),
                    };
                }
                Token::Question => {
                    // Nullable access
                    self.advance();
                    if matches!(self.current(), Token::Dot) {
                        self.advance();
                        let field = match self.advance() {
                            Token::Ident(id) => id,
                            _ => {
                                return Err(OmniError::ParserError {
                                    location: self.location(),
                                    message: "Expected field name".to_string(),
                                });
                            }
                        };
                        // For now, just treat as regular field access
                        expr = Expr::FieldAccess {
                            object: Box::new(expr),
                            field,
                            span: self.span(start),
                        };
                    }
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr> {
        let start = self.position();
        
        match self.current().clone() {
            Token::Int(n) => {
                self.advance();
                Ok(Expr::Literal(Literal::Int(n)))
            }
            Token::Float(n) => {
                self.advance();
                Ok(Expr::Literal(Literal::Float(n)))
            }
            Token::Bool(b) => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(b)))
            }
            Token::String(s) => {
                self.advance();
                Ok(Expr::Literal(Literal::String(s)))
            }
            Token::Char(c) => {
                self.advance();
                Ok(Expr::Literal(Literal::Char(c)))
            }
            Token::Ident(id) => {
                self.advance();
                Ok(Expr::Ident(id))
            }
            Token::KwTrue => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            Token::KwFalse => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            Token::KwNull => {
                self.advance();
                Ok(Expr::Literal(Literal::Int(0))) // Represent null as 0 for now
            }
            Token::LParen => {
                self.advance();
                // Tuple or grouped expression
                if matches!(self.current(), Token::RParen) {
                    self.advance();
                    return Ok(Expr::Tuple(Vec::new(), self.span(start)));
                }
                
                let expr = self.parse_expression()?;
                
                if matches!(self.current(), Token::Comma) {
                    // Tuple
                    let mut elements = vec![expr];
                    while matches!(self.current(), Token::Comma) {
                        self.advance();
                        elements.push(self.parse_expression()?);
                    }
                    self.expect(Token::RParen)?;
                    Ok(Expr::Tuple(elements, self.span(start)))
                } else {
                    self.expect(Token::RParen)?;
                    Ok(expr)
                }
            }
            Token::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                while !matches!(self.current(), Token::RBracket) {
                    elements.push(self.parse_expression()?);
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Expr::Array(elements, self.span(start)))
            }
            Token::LBrace => {
                // Could be a struct literal or a block
                self.advance();
                
                // Check if it's a struct literal
                if let Token::Ident(name) = self.current().clone() {
                    self.advance();
                    if matches!(self.current(), Token::LBrace) {
                        // Struct literal
                        self.advance();
                        let mut fields = Vec::new();
                        while !matches!(self.current(), Token::RBrace) {
                            let field_name = match self.advance() {
                                Token::Ident(id) => id,
                                _ => {
                                    return Err(OmniError::ParserError {
                                        location: self.location(),
                                        message: "Expected field name".to_string(),
                                    });
                                }
                            };
                            self.expect(Token::Colon)?;
                            let field_value = self.parse_expression()?;
                            fields.push((field_name, field_value));
                            if matches!(self.current(), Token::Comma) {
                                self.advance();
                            }
                        }
                        self.expect(Token::RBrace)?;
                        return Ok(Expr::StructLiteral {
                            name,
                            fields,
                            span: self.span(start),
                        });
                    }
                }
                
                // It's a block
                let (stmts, expr) = self.parse_block_with_expr(Token::RBrace)?;
                Ok(Expr::Block {
                    statements: stmts,
                    expr: expr.map(Box::new),
                    span: self.span(start),
                })
            }
            Token::KwIf => {
                self.advance();
                let condition = self.parse_expression()?;
                self.expect(Token::KwElse)?;
                let then_branch = Box::new(self.parse_expression()?());
                let else_branch = Box::new(self.parse_expression()?);
                Ok(Expr::If {
                    condition: Box::new(condition),
                    then_branch,
                    else_branch,
                    span: self.span(start),
                })
            }
            Token::KwMatch => {
                self.advance();
                let expr = Box::new(self.parse_expression()?);
                self.expect(Token::LBrace)?;
                
                let mut arms = Vec::new();
                while !matches!(self.current(), Token::RBrace) {
                    let pattern = self.parse_pattern()?;
                    self.expect(Token::FatArrow)?;
                    let body = Box::new(self.parse_expression()?);
                    
                    arms.push(MatchArm {
                        pattern,
                        guard: None,
                        body: vec![Stmt::Return {
                            value: Some(body),
                            span: Span::dummy(),
                        }],
                        span: Span::dummy(),
                    });
                    
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RBrace)?;
                
                Ok(Expr::Match {
                    expr,
                    arms,
                    span: self.span(start),
                })
            }
            Token::KwFn => {
                // Lambda expression
                self.advance();
                self.expect(Token::LParen)?;
                let mut args = Vec::new();
                while !matches!(self.current(), Token::RParen) {
                    let arg_name = match self.advance() {
                        Token::Ident(id) => id,
                        _ => {
                            return Err(OmniError::ParserError {
                                location: self.location(),
                                message: "Expected argument name".to_string(),
                            });
                        }
                    };
                    let arg_type = if matches!(self.current(), Token::Colon) {
                        self.advance();
                        self.parse_type()?
                    } else {
                        Type::Dynamic
                    };
                    args.push(Arg {
                        name: arg_name,
                        arg_type,
                        default: None,
                        span: Span::dummy(),
                    });
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RParen)?;
                
                let return_type = if matches!(self.current(), Token::RArrow) {
                    self.advance();
                    self.parse_type()?
                } else {
                    Type::Dynamic
                };
                
                let body = if matches!(self.current(), Token::LBrace) {
                    self.advance();
                    self.parse_block(Token::RBrace)?
                } else {
                    vec![self.parse_statement()?]
                };
                
                Ok(Expr::Lambda {
                    args,
                    return_type: Some(return_type),
                    body,
                    span: self.span(start),
                })
            }
            Token::Pipe => {
                // Alternative lambda syntax: |x, y| x + y
                self.advance();
                let mut args = Vec::new();
                while !matches!(self.current(), Token::Pipe) {
                    let arg_name = match self.advance() {
                        Token::Ident(id) => id,
                        _ => {
                            return Err(OmniError::ParserError {
                                location: self.location(),
                                message: "Expected argument name".to_string(),
                            });
                        }
                    };
                    args.push(Arg {
                        name: arg_name,
                        arg_type: Type::Dynamic,
                        default: None,
                        span: Span::dummy(),
                    });
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::Pipe)?;
                
                let body = if matches!(self.current(), Token::LBrace) {
                    self.advance();
                    self.parse_block(Token::RBrace)?
                } else {
                    vec![self.parse_expression()?.into()]
                };
                
                Ok(Expr::Lambda {
                    args,
                    return_type: None,
                    body,
                    span: self.span(start),
                })
            }
            _ => {
                Err(OmniError::ParserError {
                    location: self.location(),
                    message: format!("Unexpected token: {:?}", self.current()),
                })
            }
        }
    }

    fn parse_block_with_expr(&mut self, end_token: Token) -> Result<(Vec<Stmt>, Option<Expr>)> {
        let mut statements = Vec::new();
        
        while !matches!(self.current(), end_token) && !matches!(self.current(), Token::Eof) {
            if matches!(self.current(), Token::Newline | Token::Indent | Token::Dedent) {
                self.advance();
                continue;
            }
            
            // Check for trailing expression
            if matches!(self.peek(0), Token::Newline) && matches!(self.peek(1), end_token) {
                break;
            }
            
            statements.push(self.parse_statement()?);
        }
        
        // Check for trailing expression (no semicolon)
        let expr = if !statements.is_empty() {
            if let Stmt::ExprStmt(expr) = statements.last().unwrap() {
                if matches!(self.peek(0), end_token) || matches!(self.peek(0), Token::Newline) {
                    statements.pop();
                    Some(expr.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        if !matches!(self.current(), Token::Eof) {
            self.advance(); // consume end token
        }
        
        Ok((statements, expr))
    }
}

// Helper trait to convert Expr to Stmt
impl From<Expr> for Stmt {
    fn from(expr: Expr) -> Self {
        Stmt::ExprStmt(expr)
    }
}
