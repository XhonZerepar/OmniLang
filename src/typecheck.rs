//! Type Checker for OmniLang
//! 
//! Performs semantic analysis and type checking on the AST.

use crate::ast::*;
use crate::errors::{OmniError, Result};
use std::collections::{HashMap, HashSet};

/// Type checking context
pub struct TypeChecker {
    scopes: Vec<HashMap<String, Type>>,
    structs: HashMap<String, StructDef>,
    enums: HashMap<String, EnumDef>,
    functions: HashMap<String, Function>,
    current_function: Option<String>,
    errors: Vec<OmniError>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut scopes = Vec::new();
        scopes.push(HashMap::new());
        
        Self {
            scopes,
            structs: HashMap::new(),
            enums: HashMap::new(),
            functions: HashMap::new(),
            current_function: None,
            errors: Vec::new(),
        }
    }

    pub fn check(&mut self, program: &Program) -> Result<()> {
        // First pass: register all types and functions
        self.register_types(program)?;
        
        // Second pass: check function bodies
        self.check_functions(program)?;
        
        // Check statements
        for stmt in &program.statements {
            self.check_statement(stmt)?;
        }
        
        if !self.errors.is_empty() {
            return Err(self.errors.remove(0));
        }
        
        Ok(())
    }

    fn register_types(&mut self, program: &Program) -> Result<()> {
        // Register structs
        for struct_def in &program.structs {
            self.structs.insert(struct_def.name.clone(), struct_def.clone());
        }
        
        // Register enums
        for enum_def in &program.enums {
            self.enums.insert(enum_def.name.clone(), enum_def.clone());
        }
        
        // Register functions
        for func in &program.functions {
            self.functions.insert(func.name.clone(), func.clone());
        }
        
        Ok(())
    }

    fn check_functions(&mut self, program: &Program) -> Result<()> {
        for func in &program.functions {
            let prev_fn = self.current_function.replace(func.name.clone());
            
            // Create new scope for function parameters
            self.push_scope();
            for arg in &func.args {
                self.scopes.last_mut().unwrap().insert(arg.name.clone(), arg.arg_type.clone());
            }
            
            // Check function body
            for stmt in &func.body {
                self.check_statement(stmt)?;
            }
            
            self.pop_scope();
            self.current_function = prev_fn;
        }
        
        Ok(())
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn lookup_var(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t.clone());
            }
        }
        None
    }

    fn check_statement(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, var_type, value, span } => {
                let value_type = self.check_expression(value)?;
                
                // Check type annotation
                if let Some(annotated) = var_type {
                    if !self.types_compatible(annotated, &value_type) {
                        self.errors.push(OmniError::TypeError {
                            location: span.to_string(),
                            message: format!("Type mismatch: expected {:?}, got {:?}", annotated, value_type),
                        });
                    }
                }
                
                // Register variable
                self.scopes.last_mut().unwrap().insert(name.clone(), value_type);
                Ok(())
            }
            
            Stmt::Mut { name, var_type, value, span } => {
                let value_type = self.check_expression(value)?;
                
                if let Some(annotated) = var_type {
                    if !self.types_compatible(annotated, &value_type) {
                        self.errors.push(OmniError::TypeError {
                            location: span.to_string(),
                            message: format!("Type mismatch: expected {:?}, got {:?}", annotated, value_type),
                        });
                    }
                }
                
                self.scopes.last_mut().unwrap().insert(name.clone(), value_type);
                Ok(())
            }
            
            Stmt::Assign { target, value, span } => {
                let target_type = self.check_expression(target)?;
                let value_type = self.check_expression(value)?;
                
                if !self.types_compatible(&target_type, &value_type) {
                    self.errors.push(OmniError::TypeError {
                        location: span.to_string(),
                        message: format!("Cannot assign {:?} to {:?}", value_type, target_type),
                    });
                }
                
                Ok(())
            }
            
            Stmt::ExprStmt(expr) => {
                self.check_expression(expr)?;
                Ok(())
            }
            
            Stmt::Return { value, span } => {
                if let Some(expr) = value {
                    let ret_type = self.check_expression(expr)?;
                    
                    // Check against function return type
                    if let Some(fn_name) = &self.current_function {
                        if let Some(func) = self.functions.get(fn_name) {
                            if !self.types_compatible(&func.return_type, &ret_type) {
                                self.errors.push(OmniError::TypeError {
                                    location: span.to_string(),
                                    message: format!("Return type mismatch: expected {:?}, got {:?}", func.return_type, ret_type),
                                });
                            }
                        }
                    }
                }
                Ok(())
            }
            
            Stmt::If { condition, then_branch, else_branch, span } => {
                let cond_type = self.check_expression(condition)?;
                if !matches!(cond_type, Type::Bool) {
                    self.errors.push(OmniError::TypeError {
                        location: span.to_string(),
                        message: format!("If condition must be Bool, got {:?}", cond_type),
                    });
                }
                
                self.push_scope();
                for stmt in then_branch {
                    self.check_statement(stmt)?;
                }
                self.pop_scope();
                
                if let Some(else_branch) = else_branch {
                    self.push_scope();
                    for stmt in else_branch {
                        self.check_statement(stmt)?;
                    }
                    self.pop_scope();
                }
                
                Ok(())
            }
            
            Stmt::Match { expr, arms, span } => {
                let expr_type = self.check_expression(expr)?;
                
                for arm in arms {
                    self.check_pattern(&arm.pattern, &expr_type)?;
                    
                    if let Some(guard) = &arm.guard {
                        let guard_type = self.check_expression(guard)?;
                        if !matches!(guard_type, Type::Bool) {
                            self.errors.push(OmniError::TypeError {
                                location: span.to_string(),
                                message: "Match guard must be Bool".to_string(),
                            });
                        }
                    }
                    
                    self.push_scope();
                    for stmt in &arm.body {
                        self.check_statement(stmt)?;
                    }
                    self.pop_scope();
                }
                
                Ok(())
            }
            
            Stmt::Loop { body, span } => {
                self.push_scope();
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.pop_scope();
                Ok(())
            }
            
            Stmt::While { condition, body, span } => {
                let cond_type = self.check_expression(condition)?;
                if !matches!(cond_type, Type::Bool) {
                    self.errors.push(OmniError::TypeError {
                        location: span.to_string(),
                        message: "While condition must be Bool".to_string(),
                    });
                }
                
                self.push_scope();
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.pop_scope();
                Ok(())
            }
            
            Stmt::For { variable, iter, body, span } => {
                let iter_type = self.check_expression(iter)?;
                
                // Register loop variable
                self.push_scope();
                let elem_type = match &iter_type {
                    Type::Vec(t) => *t.clone(),
                    Type::Array(t, _) => *t.clone(),
                    Type::HashSet(t) => *t.clone(),
                    _ => Type::Dynamic,
                };
                self.scopes.last_mut().unwrap().insert(variable.clone(), elem_type);
                
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.pop_scope();
                Ok(())
            }
            
            Stmt::Break { .. } => Ok(()),
            Stmt::Continue { .. } => Ok(()),
            
            Stmt::Block { statements, span } => {
                self.push_scope();
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
                self.pop_scope();
                Ok(())
            }
            
            Stmt::Unsafe { body, span } => {
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                Ok(())
            }
            
            Stmt::Async { body, span } => {
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                Ok(())
            }
            
            Stmt::Try { body, catches, finally, span } => {
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                
                for catch in catches {
                    self.push_scope();
                    if let Some(var) = &catch.variable {
                        self.scopes.last_mut().unwrap().insert(var.clone(), Type::Dynamic);
                    }
                    for stmt in &catch.body {
                        self.check_statement(stmt)?;
                    }
                    self.pop_scope();
                }
                
                if let Some(finally) = finally {
                    for stmt in finally {
                        self.check_statement(stmt)?;
                    }
                }
                
                Ok(())
            }
            
            Stmt::Throw { expr, span } => {
                self.check_expression(expr)?;
                Ok(())
            }
            
            Stmt::AugAssign { target, op, value, span } => {
                let target_type = self.check_expression(target)?;
                let value_type = self.check_expression(value)?;
                
                if !target_type.is_numeric() {
                    self.errors.push(OmniError::TypeError {
                        location: span.to_string(),
                        message: format!("Cannot use augmented assignment on non-numeric type {:?}", target_type),
                    });
                }
                
                Ok(())
            }
        }
    }

    fn check_expression(&mut self, expr: &Expr) -> Result<Type> {
        match expr {
            Expr::Literal(lit) => Ok(lit.get_type()),
            
            Expr::Ident(name) => {
                if let Some(t) = self.lookup_var(name) {
                    Ok(t)
                } else if let Some(func) = self.functions.get(name) {
                    Ok(Type::Function(
                        func.args.iter().map(|a| a.arg_type.clone()).collect(),
                        Box::new(func.return_type.clone())
                    ))
                } else {
                    self.errors.push(OmniError::TypeError {
                        location: "<unknown>".to_string(),
                        message: format!("Unknown identifier: {}", name),
                    });
                    Ok(Type::Dynamic)
                }
            }
            
            Expr::Binary { left, op, right, span } => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;
                
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::Pow => {
                        if left_type.is_numeric() && right_type.is_numeric() {
                            Ok(left_type)
                        } else {
                            self.errors.push(OmniError::TypeError {
                                location: span.to_string(),
                                message: format!("Cannot perform arithmetic on {:?} and {:?}", left_type, right_type),
                            });
                            Ok(Type::Dynamic)
                        }
                    }
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        if left_type.is_numeric() && right_type.is_numeric() {
                            Ok(Type::Bool)
                        } else if self.types_compatible(&left_type, &right_type) {
                            Ok(Type::Bool)
                        } else {
                            self.errors.push(OmniError::TypeError {
                                location: span.to_string(),
                                message: format!("Cannot compare {:?} and {:?}", left_type, right_type),
                            });
                            Ok(Type::Bool)
                        }
                    }
                    BinOp::And | BinOp::Or => {
                        if matches!(left_type, Type::Bool) && matches!(right_type, Type::Bool) {
                            Ok(Type::Bool)
                        } else {
                            self.errors.push(OmniError::TypeError {
                                location: span.to_string(),
                                message: "Logical operators require Bool operands".to_string(),
                            });
                            Ok(Type::Bool)
                        }
                    }
                    BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => {
                        if left_type.is_numeric() && right_type.is_numeric() {
                            Ok(left_type)
                        } else {
                            self.errors.push(OmniError::TypeError {
                                location: span.to_string(),
                                message: "Bitwise operators require numeric operands".to_string(),
                            });
                            Ok(Type::Dynamic)
                        }
                    }
                    BinOp::Shl | BinOp::Shr => {
                        if left_type.is_numeric() && right_type.is_numeric() {
                            Ok(left_type)
                        } else {
                            self.errors.push(OmniError::TypeError {
                                location: span.to_string(),
                                message: "Shift operators require numeric operands".to_string(),
                            });
                            Ok(Type::Dynamic)
                        }
                    }
                    _ => Ok(Type::Dynamic),
                }
            }
            
            Expr::Unary { op, operand, span } => {
                let operand_type = self.check_expression(operand)?;
                
                match op {
                    UnOp::Neg => {
                        if operand_type.is_numeric() {
                            Ok(operand_type)
                        } else {
                            self.errors.push(OmniError::TypeError {
                                location: span.to_string(),
                                message: "Cannot negate non-numeric type".to_string(),
                            });
                            Ok(Type::Dynamic)
                        }
                    }
                    UnOp::Not => Ok(Type::Bool),
                    UnOp::BitNot => {
                        if operand_type.is_numeric() {
                            Ok(operand_type)
                        } else {
                            self.errors.push(OmniError::TypeError {
                                location: span.to_string(),
                                message: "Cannot bitwise negate non-numeric type".to_string(),
                            });
                            Ok(Type::Dynamic)
                        }
                    }
                    UnOp::Ref => Ok(Type::Ref(Box::new(operand_type))),
                    UnOp::MutRef => Ok(Type::MutRef(Box::new(operand_type))),
                    UnOp::Deref => {
                        if let Type::Ptr(t) = &operand_type {
                            Ok(*t.clone())
                        } else {
                            self.errors.push(OmniError::TypeError {
                                location: span.to_string(),
                                message: "Cannot dereference non-pointer type".to_string(),
                            });
                            Ok(Type::Dynamic)
                        }
                    }
                }
            }
            
            Expr::Call { func, args, span } => {
                let func_type = self.check_expression(func)?;
                
                if let Type::Function(param_types, return_type) = &func_type {
                    // Check argument count
                    if param_types.len() != args.len() {
                        self.errors.push(OmniError::TypeError {
                            location: span.to_string(),
                            message: format!("Expected {} arguments, got {}", param_types.len(), args.len()),
                        });
                    }
                    
                    // Check argument types
                    for (i, (arg, expected)) in args.iter().zip(param_types.iter()).enumerate() {
                        let arg_type = self.check_expression(arg)?;
                        if !self.types_compatible(expected, &arg_type) {
                            self.errors.push(OmniError::TypeError {
                                location: span.to_string(),
                                message: format!("Argument {} type mismatch: expected {:?}, got {:?}", i, expected, arg_type),
                            });
                        }
                    }
                    
                    Ok(*return_type.clone())
                } else {
                    self.errors.push(OmniError::TypeError {
                        location: span.to_string(),
                        message: format!("Cannot call non-function type {:?}", func_type),
                    });
                    Ok(Type::Dynamic)
                }
            }
            
            Expr::MethodCall { object, method, args, span } => {
                let obj_type = self.check_expression(object)?;
                
                // For now, just return dynamic type for method calls
                // A full implementation would look up the method in the type
                for arg in args {
                    self.check_expression(arg)?;
                }
                
                Ok(Type::Dynamic)
            }
            
            Expr::Index { object, index, span } => {
                let obj_type = self.check_expression(object)?;
                let _index_type = self.check_expression(index)?;
                
                match &obj_type {
                    Type::Vec(t) => Ok(*t.clone()),
                    Type::Array(t, _) => Ok(*t.clone()),
                    Type::HashMap(k, v) => Ok(*v.clone()),
                    Type::String => Ok(Type::Char),
                    _ => {
                        self.errors.push(OmniError::TypeError {
                            location: span.to_string(),
                            message: format!("Cannot index type {:?}", obj_type),
                        });
                        Ok(Type::Dynamic)
                    }
                }
            }
            
            Expr::FieldAccess { object, field, span } => {
                let obj_type = self.check_expression(object)?;
                
                // Look up field in struct
                if let Type::Custom(name) = &obj_type {
                    if let Some(struct_def) = self.structs.get(name) {
                        for f in &struct_def.fields {
                            if &f.name == field {
                                return Ok(f.field_type.clone());
                            }
                        }
                    }
                }
                
                Ok(Type::Dynamic)
            }
            
            Expr::Tuple(elements, span) => {
                let types: Vec<Type> = elements
                    .iter()
                    .map(|e| self.check_expression(e))
                    .collect::<Result<Vec<_>>>()?;
                
                // For now, just return Dynamic for tuples
                Ok(Type::Dynamic)
            }
            
            Expr::Array(elements, span) => {
                if elements.is_empty() {
                    return Ok(Type::Vec(Box::new(Type::Dynamic)));
                }
                
                let elem_type = self.check_expression(&elements[0])?;
                Ok(Type::Vec(Box::new(elem_type)))
            }
            
            Expr::StructLiteral { name, fields, span } => {
                if let Some(struct_def) = self.structs.get(name) {
                    // Check all required fields are present
                    for field in &struct_def.fields {
                        if !fields.iter().any(|(f, _)| f == &field.name) {
                            self.errors.push(OmniError::TypeError {
                                location: span.to_string(),
                                message: format!("Missing field '{}'", field.name),
                            });
                        }
                    }
                    
                    Ok(Type::Custom(name.clone()))
                } else {
                    self.errors.push(OmniError::TypeError {
                        location: span.to_string(),
                        message: format!("Unknown struct '{}'", name),
                    });
                    Ok(Type::Dynamic)
                }
            }
            
            Expr::Lambda { args, return_type, body, span } => {
                self.push_scope();
                for arg in args {
                    self.scopes.last_mut().unwrap().insert(arg.name.clone(), arg.arg_type.clone());
                }
                
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.pop_scope();
                
                let inferred_return = return_type.clone().unwrap_or(Type::Dynamic);
                Ok(Type::Function(
                    args.iter().map(|a| a.arg_type.clone()).collect(),
                    Box::new(inferred_return)
                ))
            }
            
            Expr::If { condition, then_branch, else_branch, span } => {
                let cond_type = self.check_expression(condition)?;
                if !matches!(cond_type, Type::Bool) {
                    self.errors.push(OmniError::TypeError {
                        location: span.to_string(),
                        message: "If condition must be Bool".to_string(),
                    });
                }
                
                let then_type = self.check_expression(then_branch)?;
                let else_type = self.check_expression(else_branch)?;
                
                // Both branches should have compatible types
                Ok(then_type)
            }
            
            Expr::Match { expr, arms, span } => {
                let _expr_type = self.check_expression(expr)?;
                
                // Check all arms return the same type
                let mut result_type = Type::Dynamic;
                for arm in arms {
                    for stmt in &arm.body {
                        self.check_statement(stmt)?;
                    }
                }
                
                Ok(result_type)
            }
            
            Expr::Block { statements, expr, span } => {
                self.push_scope();
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
                
                let result_type = if let Some(expr) = expr {
                    self.check_expression(expr)?
                } else {
                    Type::Void
                };
                
                self.pop_scope();
                Ok(result_type)
            }
            
            Expr::Cast { expr, target_type, span } => {
                self.check_expression(expr)?;
                Ok(target_type.clone())
            }
            
            Expr::Range { start, end, inclusive, span } => {
                self.check_expression(start)?;
                self.check_expression(end)?;
                Ok(Type::Vec(Box::new(Type::Int)))
            }
            
            Expr::Await { expr, span } => {
                self.check_expression(expr)
            }
            
            Expr::Move { expr, span } => {
                self.check_expression(expr)
            }
            
            Expr::Borrow { mutable, expr, span } => {
                let expr_type = self.check_expression(expr)?;
                if *mutable {
                    Ok(Type::MutRef(Box::new(expr_type)))
                } else {
                    Ok(Type::Ref(Box::new(expr_type)))
                }
            }
            
            Expr::Foreign { language, code, span } => {
                // Foreign code returns dynamic type
                Ok(Type::Dynamic)
            }
            
            Expr::Try { expr, span } => {
                self.check_expression(expr)
            }
            
            Expr::EnumLiteral { name, variant, data, span } => {
                if let Some(enum_def) = self.enums.get(name) {
                    for v in &enum_def.variants {
                        if &v.name == variant {
                            return Ok(Type::Custom(name.clone()));
                        }
                    }
                    self.errors.push(OmniError::TypeError {
                        location: span.to_string(),
                        message: format!("Unknown variant '{}' of enum '{}'", variant, name),
                    });
                } else {
                    self.errors.push(OmniError::TypeError {
                        location: span.to_string(),
                        message: format!("Unknown enum '{}'", name),
                    });
                }
                Ok(Type::Dynamic)
            }
        }
    }

    fn check_pattern(&mut self, pattern: &Pat, expected_type: &Type) -> Result<()> {
        match pattern {
            Pat::Wildcard => Ok(()),
            Pat::Lit(lit) => {
                let lit_type = lit.get_type();
                if self.types_compatible(expected_type, &lit_type) {
                    Ok(())
                } else {
                    self.errors.push(OmniError::TypeError {
                        location: "<unknown>".to_string(),
                        message: format!("Pattern type {:?} doesn't match expected {:?}", lit_type, expected_type),
                    });
                    Ok(())
                }
            }
            Pat::Ident(name) => {
                self.scopes.last_mut().unwrap().insert(name.clone(), expected_type.clone());
                Ok(())
            }
            Pat::Tuple(patterns) => {
                for pat in patterns {
                    self.check_pattern(pat, expected_type)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual {
            return true;
        }
        
        // Allow dynamic type to be compatible with anything
        if matches!(expected, Type::Dynamic) || matches!(actual, Type::Dynamic) {
            return true;
        }
        
        // Allow numeric conversions
        if expected.is_numeric() && actual.is_numeric() {
            return true;
        }
        
        false
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
