//! LLVM Code Generator for OmniLang
//! 
//! Transforms the AST into LLVM IR for native code generation.

use crate::ast::*;
use crate::errors::{OmniError, Result};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::file::Module;
use inkwell::instrumentation::InstrumentationMap;
use inkwell::intrinsics::Intrinsic;
use inkwell::module::Linkage;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::types::{AnyTypeEnum, BasicTypeEnum, FunctionType, IntType, PointerType, StructType};
use inkwell::values::{
    AnyValue, AnyValueEnum, BasicValueEnum, CallableValue, FunctionValue, IntValue, PointerValue,
    StructValue,
};
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;
use std::collections::HashMap;

pub struct CodeGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    
    // Type mappings
    int_type: IntType<'ctx>,
    float_type: BasicTypeEnum<'ctx>,
    bool_type: BasicTypeEnum<'ctx>,
    char_type: IntType<'ctx>,
    string_type: PointerType<'ctx>,
    void_type: AnyTypeEnum<'ctx>,
    
    // Symbol table
    variables: HashMap<String, PointerValue<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
    structs: HashMap<String, StructType<'ctx>>,
    
    // Current function
    current_function: Option<FunctionValue<'ctx>>,
}

impl<'ctx> CodeGenerator<'ctx> {
    pub fn new(context: &'ctx Context, name: &str) -> Result<Self> {
        let module = context.create_module(name);
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| OmniError::CodegenError { message: e.to_string() })?;
        
        let builder = context.create_builder();
        
        let int_type = context.i64_type();
        let float_type = context.f64_type();
        let bool_type = context.bool_type();
        let char_type = context.i8_type();
        let string_type = context.i8_type().ptr_type(AddressSpace::Const);
        let void_type = context.void_type();
        
        Ok(Self {
            context,
            module,
            builder,
            execution_engine,
            int_type,
            float_type,
            bool_type,
            char_type,
            string_type,
            void_type,
            variables: HashMap::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            current_function: None,
        })
    }

    pub fn compile(&mut self, program: &Program) -> Result<()> {
        // Initialize types
        self.init_types()?;
        
        // Generate functions
        for func in &program.functions {
            self.generate_function(func)?;
        }
        
        // Generate top-level statements
        for stmt in &program.statements {
            self.generate_statement(stmt)?;
        }
        
        // Create main function if not exists
        if !self.functions.contains_key("main") {
            self.create_main_function()?;
        }
        
        Ok(())
    }

    fn init_types(&mut self) -> Result<()> {
        // Printf function
        let printf_type = self.context.i32_type().fn_type(&[self.string_type.into()], true);
        self.module.add_function("printf", printf_type, Some(Linkage::External));
        
        // Malloc and free
        let malloc_type = self.int_type.fn_type(&[self.int_type.into()], false);
        self.module.add_function("malloc", malloc_type, Some(Linkage::External));
        
        let free_type = self.context.i32_type().fn_type(&[self.string_type.into()], false);
        self.module.add_function("free", free_type, Some(Linkage::External));
        
        Ok(())
    }

    fn create_main_function(&mut self) -> Result<()> {
        let main_type = self.context.i32_type().fn_type(&[], false);
        let main_func = self.module.add_function("main", main_type, Some(Linkage::External));
        
        let entry = self.context.append_basic_block(main_func, "entry");
        self.builder.position_at_end(entry);
        
        self.current_function = Some(main_func);
        
        // Return 0
        let zero = self.int_type.const_int(0, false);
        self.builder.build_return(Some(&zero));
        
        self.current_function = None;
        
        Ok(())
    }

    fn generate_function(&mut self, func: &Function) -> Result<()> {
        // Create function type
        let return_type = self.map_type(&func.return_type);
        let param_types: Vec<BasicTypeEnum> = func
            .args
            .iter()
            .map(|arg| self.map_type(&arg.arg_type).into())
            .collect();
        
        let fn_type = return_type.fn_type(&param_types, false);
        let function = self.module.add_function(&func.name, fn_type, Some(Linkage::External));
        
        self.functions.insert(func.name.clone(), function);
        
        // Generate function body
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);
        
        // Clear variables for new function
        self.variables.clear();
        
        // Store arguments in variables
        for (i, arg) in func.args.iter().enumerate() {
            let alloca = self.builder.build_alloca(
                self.map_type(&arg.arg_type).into(),
                &arg.name,
            );
            let param = function.get_nth_param(i as u32).unwrap();
            self.builder.build_store(alloca, param);
            self.variables.insert(arg.name.clone(), alloca);
        }
        
        let prev_fn = self.current_function.replace(function);
        
        // Generate body statements
        for stmt in &func.body {
            self.generate_statement(stmt)?;
        }
        
        // Add default return if needed
        if !self.builder.get_insert_block().map(|b| b.get_terminator()).is_some() {
            self.builder.build_return(Some(&self.map_type(&func.return_type).into_const(&self.int_type.const_int(0, false))));
        }
        
        self.current_function = prev_fn;
        
        Ok(())
    }

    fn generate_statement(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, var_type, value, span } => {
                let value = self.generate_expression(value)?;
                let alloca = self.builder.build_alloca(
                    value.get_type(),
                    name,
                );
                self.builder.build_store(alloca, value);
                self.variables.insert(name.clone(), alloca);
                Ok(())
            }
            
            Stmt::Mut { name, var_type, value, span } => {
                let value = self.generate_expression(value)?;
                if let Some(alloca) = self.variables.get(name) {
                    self.builder.build_store(*alloca, value);
                }
                Ok(())
            }
            
            Stmt::Return { value, span } => {
                if let Some(expr) = value {
                    let value = self.generate_expression(expr)?;
                    self.builder.build_return(Some(&value));
                } else {
                    self.builder.build_return(None);
                }
                Ok(())
            }
            
            Stmt::ExprStmt(expr) => {
                self.generate_expression(expr)?;
                Ok(())
            }
            
            Stmt::If { condition, then_branch, else_branch, span } => {
                let cond = self.generate_expression(condition)?;
                let cond_int = self.builder.build_int_cast(
                    cond.into_int_value(),
                    self.bool_type.into(),
                    "cond_cast",
                );
                
                let then_block = self.context.append_basic_block(
                    self.current_function.unwrap(),
                    "then",
                );
                let else_block = self.context.append_basic_block(
                    self.current_function.unwrap(),
                    "else",
                );
                let merge_block = self.context.append_basic_block(
                    self.current_function.unwrap(),
                    "merge",
                );
                
                self.builder.build_conditional_branch(cond_int, then_block, else_block);
                
                // Then block
                self.builder.position_at_end(then_block);
                for stmt in then_branch {
                    self.generate_statement(stmt)?;
                }
                if !self.builder.get_insert_block().map(|b| b.get_terminator()).is_some() {
                    self.builder.build_unconditional_branch(merge_block);
                }
                
                // Else block
                self.builder.position_at_end(else_block);
                if let Some(else_branch) = else_branch {
                    for stmt in else_branch {
                        self.generate_statement(stmt)?;
                    }
                }
                if !self.builder.get_insert_block().map(|b| b.get_terminator()).is_some() {
                    self.builder.build_unconditional_branch(merge_block);
                }
                
                // Merge block
                self.builder.position_at_end(merge_block);
                
                Ok(())
            }
            
            Stmt::Loop { body, span } => {
                let loop_block = self.context.append_basic_block(
                    self.current_function.unwrap(),
                    "loop",
                );
                let end_block = self.context.append_basic_block(
                    self.current_function.unwrap(),
                    "loop_end",
                );
                
                self.builder.build_unconditional_branch(loop_block);
                self.builder.position_at_end(loop_block);
                
                for stmt in body {
                    self.generate_statement(stmt)?;
                }
                
                if !self.builder.get_insert_block().map(|b| b.get_terminator()).is_some() {
                    self.builder.build_unconditional_branch(loop_block);
                }
                
                self.builder.position_at_end(end_block);
                
                Ok(())
            }
            
            Stmt::While { condition, body, span } => {
                let cond_block = self.context.append_basic_block(
                    self.current_function.unwrap(),
                    "while_cond",
                );
                let body_block = self.context.append_basic_block(
                    self.current_function.unwrap(),
                    "while_body",
                );
                let end_block = self.context.append_basic_block(
                    self.current_function.unwrap(),
                    "while_end",
                );
                
                self.builder.build_unconditional_branch(cond_block);
                
                self.builder.position_at_end(cond_block);
                let cond = self.generate_expression(condition)?;
                self.builder.build_conditional_branch(cond.into_int_value(), body_block, end_block);
                
                self.builder.position_at_end(body_block);
                for stmt in body {
                    self.generate_statement(stmt)?;
                }
                if !self.builder.get_insert_block().map(|b| b.get_terminator()).is_some() {
                    self.builder.build_unconditional_branch(cond_block);
                }
                
                self.builder.position_at_end(end_block);
                
                Ok(())
            }
            
            Stmt::Break { .. } => {
                // For now, just return
                if let Some(func) = self.current_function {
                    let end_block = self.context.append_basic_block(func, "break");
                    self.builder.build_unconditional_branch(end_block);
                    self.builder.position_at_end(end_block);
                }
                Ok(())
            }
            
            Stmt::Continue { .. } => {
                // For now, just return
                if let Some(func) = self.current_function {
                    let loop_block = self.context.append_basic_block(func, "continue");
                    self.builder.build_unconditional_branch(loop_block);
                    self.builder.position_at_end(loop_block);
                }
                Ok(())
            }
            
            Stmt::Block { statements, span } => {
                for stmt in statements {
                    self.generate_statement(stmt)?;
                }
                Ok(())
            }
            
            Stmt::For { variable, iter, body, span } => {
                // Simple for loop: for i in 0..n
                let start = self.generate_expression(&iter)?;
                // For now, just generate body once
                for stmt in body {
                    self.generate_statement(stmt)?;
                }
                Ok(())
            }
            
            _ => Ok(()), // Other statements not implemented yet
        }
    }

    fn generate_expression(&self, expr: &Expr) -> Result<BasicValueEnum<'ctx>> {
        match expr {
            Expr::Literal(lit) => self.generate_literal(lit),
            Expr::Ident(name) => {
                if let Some(alloca) = self.variables.get(name) {
                    Ok(self.builder.build_load(*alloca, name))
                } else if let Some(func) = self.functions.get(name) {
                    Ok(func.as_any_value_enum().into())
                } else {
                    Err(OmniError::CodegenError {
                        message: format!("Unknown variable: {}", name),
                    })
                }
            }
            
            Expr::Binary { left, op, right, span } => {
                let left = self.generate_expression(left)?;
                let right = self.generate_expression(right)?;
                
                match op {
                    BinOp::Add => {
                        if left.get_type().is_int_type() {
                            Ok(self.builder.build_int_add(left.into_int_value(), right.into_int_value(), "add").into())
                        } else {
                            Ok(self.builder.build_float_add(left.into_float_value(), right.into_float_value(), "fadd").into())
                        }
                    }
                    BinOp::Sub => {
                        if left.get_type().is_int_type() {
                            Ok(self.builder.build_int_sub(left.into_int_value(), right.into_int_value(), "sub").into())
                        } else {
                            Ok(self.builder.build_float_sub(left.into_float_value(), right.into_float_value(), "fsub").into())
                        }
                    }
                    BinOp::Mul => {
                        if left.get_type().is_int_type() {
                            Ok(self.builder.build_int_mul(left.into_int_value(), right.into_int_value(), "mul").into())
                        } else {
                            Ok(self.builder.build_float_mul(left.into_float_value(), right.into_float_value(), "fmul").into())
                        }
                    }
                    BinOp::Div => {
                        if left.get_type().is_int_type() {
                            Ok(self.builder.build_int_signed_div(left.into_int_value(), right.into_int_value(), "div").into())
                        } else {
                            Ok(self.builder.build_float_div(left.into_float_value(), right.into_float_value(), "fdiv").into())
                        }
                    }
                    BinOp::Mod => {
                        if left.get_type().is_int_type() {
                            Ok(self.builder.build_int_signed_rem(left.into_int_value(), right.into_int_value(), "rem").into())
                        } else {
                            Ok(self.builder.build_float_rem(left.into_float_value(), right.into_float_value(), "frem").into())
                        }
                    }
                    BinOp::Eq => {
                        if left.get_type().is_int_type() {
                            Ok(self.builder.build_int_compare(
                                inkwell::IntPredicate::EQ,
                                left.into_int_value(),
                                right.into_int_value(),
                                "eq",
                            ).into())
                        } else {
                            Ok(self.builder.build_float_compare(
                                inkwell::FloatPredicate::OEQ,
                                left.into_float_value(),
                                right.into_float_value(),
                                "eq",
                            ).into())
                        }
                    }
                    BinOp::Ne => {
                        if left.get_type().is_int_type() {
                            Ok(self.builder.build_int_compare(
                                inkwell::IntPredicate::NE,
                                left.into_int_value(),
                                right.into_int_value(),
                                "ne",
                            ).into())
                        } else {
                            Ok(self.builder.build_float_compare(
                                inkwell::FloatPredicate::ONE,
                                left.into_float_value(),
                                right.into_float_value(),
                                "ne",
                            ).into())
                        }
                    }
                    BinOp::Lt => {
                        if left.get_type().is_int_type() {
                            Ok(self.builder.build_int_compare(
                                inkwell::IntPredicate::SLT,
                                left.into_int_value(),
                                right.into_int_value(),
                                "lt",
                            ).into())
                        } else {
                            Ok(self.builder.build_float_compare(
                                inkwell::FloatPredicate::OLT,
                                left.into_float_value(),
                                right.into_float_value(),
                                "lt",
                            ).into())
                        }
                    }
                    BinOp::Gt => {
                        if left.get_type().is_int_type() {
                            Ok(self.builder.build_int_compare(
                                inkwell::IntPredicate::SGT,
                                left.into_int_value(),
                                right.into_int_value(),
                                "gt",
                            ).into())
                        } else {
                            Ok(self.builder.build_float_compare(
                                inkwell::FloatPredicate::OGT,
                                left.into_float_value(),
                                right.into_float_value(),
                                "gt",
                            ).into())
                        }
                    }
                    BinOp::And => {
                        Ok(self.builder.build_and(left.into_int_value(), right.into_int_value(), "and").into())
                    }
                    BinOp::Or => {
                        Ok(self.builder.build_or(left.into_int_value(), right.into_int_value(), "or").into())
                    }
                    _ => Err(OmniError::CodegenError {
                        message: format!("Unsupported binary operator: {:?}", op),
                    })
                }
            }
            
            Expr::Unary { op, operand, span } => {
                let operand = self.generate_expression(operand)?;
                
                match op {
                    UnOp::Neg => {
                        if operand.get_type().is_int_type() {
                            Ok(self.builder.build_int_neg(operand.into_int_value(), "neg").into())
                        } else {
                            Ok(self.builder.build_float_neg(operand.into_float_value(), "fneg").into())
                        }
                    }
                    UnOp::Not => {
                        Ok(self.builder.build_not(operand.into_int_value(), "not").into())
                    }
                    UnOp::Deref => {
                        // Load from pointer
                        let ptr = operand.into_pointer_value();
                        Ok(self.builder.build_load(ptr, "deref"))
                    }
                    UnOp::Ref => {
                        // Return the pointer value
                        Ok(operand)
                    }
                    _ => Err(OmniError::CodegenError {
                        message: format!("Unsupported unary operator: {:?}", op),
                    })
                }
            }
            
            Expr::Call { func, args, span } => {
                let func_val = self.generate_expression(func)?;
                let callable = CallableValue::try_from(func_val).map_err(|_| OmniError::CodegenError {
                    message: "Cannot call non-function".to_string(),
                })?;
                
                let args: Result<Vec<BasicValueEnum>> = args
                    .iter()
                    .map(|a| self.generate_expression(a))
                    .collect();
                
                let call_result = self.builder.build_call(callable, &args?, "call");
                Ok(call_result.try_as_basic_value().unwrap_or(self.int_type.const_int(0, false).into()))
            }
            
            Expr::String(s) => {
                // Create global string
                let global = self.module.add_global(
                    self.string_type,
                    Some(AddressSpace::Const),
                    "",
                );
                let chars: Vec<IntValue> = s
                    .chars()
                    .map(|c| self.char_type.const_int(c as u64, false))
                    .collect();
                let mut char_array = self.char_type.array_type((s.len() + 1) as u32).const_array(&chars);
                global.set_initializer(&char_array);
                global.set_constant(true);
                
                Ok(global.as_basic_value_enum())
            }
            
            _ => Err(OmniError::CodegenError {
                message: format!("Unsupported expression: {:?}", expr),
            })
        }
    }

    fn generate_literal(&self, lit: &Literal) -> Result<BasicValueEnum<'ctx>> {
        match lit {
            Literal::Int(n) => Ok(self.int_type.const_int(*n as u64, false).into()),
            Literal::Float(f) => Ok(self.float_type.const_float(*f).into()),
            Literal::Bool(b) => Ok(self.bool_type.const_int(*b as u64, false).into()),
            Literal::Char(c) => Ok(self.char_type.const_int(*c as u64, false).into()),
            Literal::String(s) => {
                let global = self.module.add_global(
                    self.string_type,
                    Some(AddressSpace::Const),
                    "",
                );
                let chars: Vec<IntValue> = s
                    .chars()
                    .map(|c| self.char_type.const_int(c as u64, false))
                    .collect();
                let char_array = self.char_type.array_type((s.len() + 1) as u32).const_array(&chars);
                global.set_initializer(&char_array);
                global.set_constant(true);
                
                Ok(global.as_basic_value_enum())
            }
            Literal::Bytes(b) => {
                let bytes: Vec<IntValue> = b.iter().map(|b| self.char_type.const_int(*b as u64, false)).collect();
                let byte_array = self.char_type.array_type(b.len() as u32).const_array(&bytes);
                
                let global = self.module.add_global(
                    byte_array.get_type().into(),
                    Some(AddressSpace::Const),
                    "",
                );
                global.set_initializer(&byte_array);
                global.set_constant(true);
                
                Ok(global.as_basic_value_enum())
            }
        }
    }

    fn map_type(&self, ast_type: &Type) -> AnyTypeEnum<'ctx> {
        match ast_type {
            Type::Int => self.int_type.into(),
            Type::Float => self.float_type.into(),
            Type::Bool => self.bool_type.into(),
            Type::Char => self.char_type.into(),
            Type::String => self.string_type.into(),
            Type::Void => self.void_type,
            Type::Ptr(t) => self.map_type(t).ptr_type(AddressSpace::Const).into(),
            Type::Ref(t) | Type::MutRef(t) => self.map_type(t).ptr_type(AddressSpace::Const).into(),
            _ => self.int_type.into(),
        }
    }

    pub fn print_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    pub fn write_bitcode(&self, path: &str) -> Result<()> {
        self.module.write_bitcode_to_path(std::path::Path::new(path)).map_err(|e| OmniError::CodegenError {
            message: e.to_string(),
        })
    }

    pub fn object_file(&self) -> Result<Vec<u8>> {
        // For JIT, we don't generate object files directly
        // This would require the target machine setup
        Err(OmniError::CodegenError {
            message: "Object file generation not implemented".to_string(),
        })
    }
}
