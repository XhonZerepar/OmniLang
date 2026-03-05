//! OmniLang Compiler (omc)
//! 
//! Command-line interface for the OmniLang compiler.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::exit;

use omnilang::{lexer::Lexer, parser::Parser as OmniParser, typecheck::TypeChecker, codegen::CodeGenerator, VERSION, NAME};

#[derive(Parser)]
#[command(name = "omc")]
#[command(about = format!("{} v{} - The Ultimate Programming Language", NAME, VERSION), long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile an OmniLang source file
    Build {
        /// Source file to compile
        file: String,
        
        /// Output executable name
        #[arg(short, long)]
        output: Option<String>,
        
        /// Print LLVM IR
        #[arg(long)]
        print_ir: bool,
        
        /// Print AST
        #[arg(long)]
        print_ast: bool,
    },
    
    /// Compile and run an OmniLang source file
    Run {
        /// Source file to run
        file: String,
        
        /// Arguments to pass to the program
        args: Vec<String>,
    },
    
    /// Print the AST of an OmniLang source file
    Ast {
        /// Source file to analyze
        file: String,
    },
    
    /// Print the LLVM IR of an OmniLang source file
    Ir {
        /// Source file to analyze
        file: String,
    },
    
    /// Format an OmniLang source file
    Format {
        /// Source file to format
        file: String,
        
        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Check an OmniLang source file for errors
    Check {
        /// Source file to check
        file: String,
    },
    
    /// Initialize a new OmniLang project
    Init {
        /// Project name
        name: String,
    },
    
    /// Package manager commands
    Package {
        /// Package command
        #[command(subcommand)]
        command: PackageCommands,
    },
}

#[derive(Subcommand)]
enum PackageCommands {
    /// Install a package
    Install {
        /// Package name
        name: String,
    },
    
    /// Build the package
    Build,
    
    /// Run tests
    Test,
    
    /// Publish package
    Publish,
}

fn main() {
    let cli = Cli::parse();
    
    // Initialize logging
    env_logger::init();
    
    match cli.command {
        Commands::Build { file, output, print_ir, print_ast } => {
            if let Err(e) = build_file(&file, output.as_deref(), print_ast, print_ir) {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
        
        Commands::Run { file, args } => {
            if let Err(e) = run_file(&file, &args) {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
        
        Commands::Ast { file } => {
            if let Err(e) = print_ast(&file) {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
        
        Commands::Ir { file } => {
            if let Err(e) = print_ir_file(&file) {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
        
        Commands::Format { file, output } => {
            if let Err(e) = format_file(&file, output.as_deref()) {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
        
        Commands::Check { file } => {
            if let Err(e) = check_file(&file) {
                eprintln!("Error: {}", e);
                exit(1);
            }
            println!("No errors found");
        }
        
        Commands::Init { name } => {
            if let Err(e) = init_project(&name) {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
        
        Commands::Package { command } => {
            if let Err(e) = handle_package_command(command) {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
    }
}

fn compile_file(file: &str, print_ast: bool, print_ir: bool) -> Result<String, String> {
    // Read source file
    let source = fs::read_to_string(file).map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Lexical analysis
    let filename = Path::new(file)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("<unknown>");
    
    let mut lexer = Lexer::new(source, filename.to_string());
    let tokens = lexer.tokenize().map_err(|e| format!("Lexer error: {}", e))?;
    
    if print_ast {
        println!("Tokens: {:?}", tokens);
    }
    
    // Parsing
    let mut parser = OmniParser::new(tokens, filename.to_string());
    let ast = parser.parse().map_err(|e| format!("Parser error: {}", e))?;
    
    if print_ast {
        println!("AST: {:#?}", ast);
    }
    
    // Type checking
    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast).map_err(|e| format!("Type error: {}", e))?;
    
    // Code generation
    let context = inkwell::context::Context::create();
    let mut codegen = CodeGenerator::new(&context, filename)
        .map_err(|e| format!("CodeGen error: {}", e))?;
    
    codegen.compile(&ast).map_err(|e| format!("CodeGen error: {}", e))?;
    
    if print_ir {
        println!("LLVM IR:\n{}", codegen.print_ir());
    }
    
    Ok(codegen.print_ir())
}

fn build_file(file: &str, output: Option<&str>, print_ast: bool, print_ir: bool) -> Result<(), String> {
    let ir = compile_file(file, print_ast, print_ir)?;
    
    // For now, just print the IR since we don't have full object file generation
    // In a full implementation, we would compile the IR to an object file and link it
    
    println!("Compilation successful!");
    
    Ok(())
}

fn run_file(file: &str, _args: &[String]) -> Result<(), String> {
    // Compile and run
    let ir = compile_file(file, false, false)?;
    
    // For JIT execution, we need to set up the execution engine
    // This is a simplified version - full implementation would JIT compile and run
    
    println!("Running {}...", file);
    println!("(JIT execution not fully implemented in this version)");
    
    Ok(())
}

fn print_ast(file: &str) -> Result<(), String> {
    compile_file(file, true, false)
}

fn print_ir_file(file: &str) -> Result<(), String> {
    compile_file(file, false, true)
}

fn format_file(file: &str, output: Option<&str>) -> Result<(), String> {
    let source = fs::read_to_string(file).map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Simple formatting - add proper indentation
    let formatted = source
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    
    let output_path = output.unwrap_or(file);
    fs::write(output_path, formatted).map_err(|e| format!("Failed to write file: {}", e))?;
    
    println!("Formatted: {}", output_path);
    
    Ok(())
}

fn check_file(file: &str) -> Result<(), String> {
    compile_file(file, false, false)?;
    Ok(())
}

fn init_project(name: &str) -> Result<(), String> {
    let dir = Path::new(name);
    
    if dir.exists() {
        return Err(format!("Directory '{}' already exists", name));
    }
    
    fs::create_dir_all(dir).map_err(|e| format!("Failed to create directory: {}", e))?;
    
    // Create main.omni
    let main_content = r#"// Welcome to OmniLang!
// This is your first OmniLang program.

fn main(args: [String]) -> Int:
    print("Hello, OmniLang!")
    return 0
"#;
    
    fs::write(dir.join("main.omni"), main_content)
        .map_err(|e| format!("Failed to create main.omni: {}", e))?;
    
    // Create omnilang.toml
    let toml_content = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#, name);
    
    fs::write(dir.join("omnilang.toml"), toml_content)
        .map_err(|e| format!("Failed to create omnilang.toml: {}", e))?;
    
    println!("Initialized OmniLang project '{}'", name);
    
    Ok(())
}

fn handle_package_command(_command: PackageCommands) -> Result<(), String> {
    // TODO: Implement package manager
    println!("Package manager not yet implemented");
    Ok(())
}
