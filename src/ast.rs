//! Abstract Syntax Tree for OmniLang
//! 
//! Defines all AST nodes that represent the structure of OmniLang programs.

use std::collections::HashMap;

/// Source location for error reporting
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub filename: String,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn dummy() -> Self {
        Span {
            start: Position::dummy(),
            end: Position::dummy(),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{} - {}:{}", 
            self.start.filename, 
            self.start.line, 
            self.start.filename, 
            self.end.line
        )
    }
}

impl Position {
    pub fn dummy() -> Self {
        Position {
            line: 0,
            column: 0,
            filename: "<unknown>".to_string(),
        }
    }
}

/// Program root node
#[derive(Debug, Clone)]
pub struct Program {
    pub imports: Vec<Import>,
    pub functions: Vec<Function>,
    pub structs: Vec<StructDef>,
    pub enums: Vec<EnumDef>,
    pub statements: Vec<Stmt>,
}

/// Import statement
#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,
    pub alias: Option<String>,
    pub items: Vec<String>,
    pub span: Span,
}

/// Function definition
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<Arg>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
    pub span: Span,
    pub is_public: bool,
    pub is_async: bool,
    pub attrs: Vec<Attribute>,
}

/// Function argument
#[derive(Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub arg_type: Type,
    pub default: Option<Expr>,
    pub span: Span,
}

/// Attribute (like #[derive], #[test], etc.)
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<AttributeArg>,
}

#[derive(Debug, Clone)]
pub enum AttributeArg {
    String(String),
    Ident(String),
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
    pub span: Span,
    pub is_public: bool,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
    pub span: Span,
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
    pub is_public: bool,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub data: Vec<Type>,
    pub span: Span,
}

/// Type definitions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // Primitive types
    Int,
    Float,
    Bool,
    Char,
    String,
    Void,
    
    // Reference types
    Ptr(Box<Type>),
    Ref(Box<Type>),
    MutRef(Box<Type>),
    
    // Composite types
    Array(Box<Type>, Option<usize>),
    Vec(Box<Type>),
    HashMap(Box<Type>, Box<Type>),
    HashSet(Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    
    // Function types
    Function(Vec<Type>, Box<Type>),
    
    // User-defined types
    Custom(String),
    
    // Special types
    Dynamic,
    Never,
}

impl Type {
    pub fn is_primitive(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::Bool | Type::Char | Type::String)
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }

    pub fn to_string(&self) -> String {
        match self {
            Type::Int => "Int".to_string(),
            Type::Float => "Float".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::Char => "Char".to_string(),
            Type::String => "String".to_string(),
            Type::Void => "Void".to_string(),
            Type::Ptr(t) => format!("*{}", t.to_string()),
            Type::Ref(t) => format!("&{}", t.to_string()),
            Type::MutRef(t) => format!("&mut {}", t.to_string()),
            Type::Array(t, size) => {
                if let Some(s) = size {
                    format!("[{}; {}]", t.to_string(), s)
                } else {
                    format!("[{}]", t.to_string())
                }
            }
            Type::Vec(t) => format!("Vec<{}>", t.to_string()),
            Type::HashMap(k, v) => format!("HashMap<{}, {}>", k.to_string(), v.to_string()),
            Type::HashSet(t) => format!("HashSet<{}>", t.to_string()),
            Type::Option(t) => format!("Option<{}>", t.to_string()),
            Type::Result(ok, err) => format!("Result<{}, {}>", ok.to_string(), err.to_string()),
            Type::Function(args, ret) => {
                let args_str = args.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ");
                format!("fn({}) -> {}", args_str, ret.to_string())
            }
            Type::Custom(name) => name.clone(),
            Type::Dynamic => "dynamic".to_string(),
            Type::Never => "never".to_string(),
        }
    }
}

/// Statement types
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Variable declaration (immutable)
    Let {
        name: String,
        var_type: Option<Type>,
        value: Expr,
        span: Span,
    },
    
    /// Variable declaration (mutable)
    Mut {
        name: String,
        var_type: Option<Type>,
        value: Expr,
        span: Span,
    },
    
    /// Assignment
    Assign {
        target: Expr,
        value: Expr,
        span: Span,
    },
    
    /// Augmented assignment (+=, -=, etc.)
    AugAssign {
        target: Expr,
        op: BinOp,
        value: Expr,
        span: Span,
    },
    
    /// Function call statement
    ExprStmt(Expr),
    
    /// Return statement
    Return {
        value: Option<Expr>,
        span: Span,
    },
    
    /// If-else statement
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
        span: Span,
    },
    
    /// Match expression
    Match {
        expr: Expr,
        arms: Vec<MatchArm>,
        span: Span,
    },
    
    /// Loop statements
    Loop {
        body: Vec<Stmt>,
        span: Span,
    },
    
    /// While loop
    While {
        condition: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    
    /// For loop
    For {
        variable: String,
        iter: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    
    /// Break statement
    Break {
        span: Span,
    },
    
    /// Continue statement
    Continue {
        span: Span,
    },
    
    /// Block statement
    Block {
        statements: Vec<Stmt>,
        span: Span,
    },
    
    /// Unsafe block
    Unsafe {
        body: Vec<Stmt>,
        span: Span,
    },
    
    /// Async block
    Async {
        body: Vec<Stmt>,
        span: Span,
    },
    
    /// Try-catch handling
    Try {
        body: Vec<Stmt>,
        catches: Vec<CatchClause>,
        finally: Option<Vec<Stmt>>,
        span: Span,
    },
    
    /// Throw statement
    Throw {
        expr: Expr,
        span: Span,
    },
}

/// Catch clause for try-catch
#[derive(Debug, Clone)]
pub struct CatchClause {
    pub exception_type: Option<Type>,
    pub variable: Option<String>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Match arm
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pat,
    pub guard: Option<Expr>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Pattern matching
#[derive(Debug, Clone)]
pub enum Pat {
    /// Wildcard pattern (_)
    Wildcard,
    
    /// Literal pattern
    Lit(Literal),
    
    /// Identifier pattern
    Ident(String),
    
    /// Tuple pattern
    Tuple(Vec<Pat>),
    
    /// Struct pattern
    Struct {
        name: String,
        fields: Vec<(String, Pat)>,
    },
    
    /// Enum pattern
    Enum {
        name: String,
        variant: String,
        data: Vec<Pat>,
    },
    
    /// Range pattern (1..5)
    Range(Box<Pat>, Box<Pat>),
    
    /// Or pattern (a | b)
    Or(Vec<Pat>),
    
    /// Slice pattern
    Slice(Vec<Pat>),
}

/// Expression types
#[derive(Debug, Clone)]
pub enum Expr {
    /// Literal values
    Literal(Literal),
    
    /// Identifier reference
    Ident(String),
    
    /// Binary operation
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
        span: Span,
    },
    
    /// Unary operation
    Unary {
        op: UnOp,
        operand: Box<Expr>,
        span: Span,
    },
    
    /// Function call
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    
    /// Method call
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
    
    /// Index access
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    
    /// Field access
    FieldAccess {
        object: Box<Expr>,
        field: String,
        span: Span,
    },
    
    /// Tuple
    Tuple(Vec<Expr>, Span),
    
    /// Array literal
    Array(Vec<Expr>, Span),
    
    /// Struct literal
    StructLiteral {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },
    
    /// Enum variant literal
    EnumLiteral {
        name: String,
        variant: String,
        data: Vec<Expr>,
        span: Span,
    },
    
    /// Lambda/anonymous function
    Lambda {
        args: Vec<Arg>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
        span: Span,
    },
    
    /// If expression
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
        span: Span,
    },
    
    /// Match expression
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    
    /// Block expression
    Block {
        statements: Vec<Stmt>,
        expr: Option<Box<Expr>>,
        span: Span,
    },
    
    /// Try expression
    Try {
        expr: Box<Expr>,
        span: Span,
    },
    
    /// Type cast
    Cast {
        expr: Box<Expr>,
        target_type: Type,
        span: Span,
    },
    
    /// Range expression
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
        span: Span,
    },
    
    /// Await expression
    Await {
        expr: Box<Expr>,
        span: Span,
    },
    
    /// Move expression
    Move {
        expr: Box<Expr>,
        span: Span,
    },
    
    /// Borrow expression
    Borrow {
        mutable: bool,
        expr: Box<Expr>,
        span: Span,
    },
    
    /// Reference to inline foreign code
    Foreign {
        language: String,
        code: String,
        span: Span,
    },
}

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    
    // Logical
    And,
    Or,
    
    // Assignment
    Assign,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Not,
    Neg,
    BitNot,
    Ref,
    MutRef,
    Deref,
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Bytes(Vec<u8>),
}

impl Literal {
    pub fn get_type(&self) -> Type {
        match self {
            Literal::Int(_) => Type::Int,
            Literal::Float(_) => Type::Float,
            Literal::Bool(_) => Type::Bool,
            Literal::Char(_) => Type::Char,
            Literal::String(_) => Type::String,
            Literal::Bytes(_) => Type::Array(Box::new(Type::Char), None),
        }
    }
}
