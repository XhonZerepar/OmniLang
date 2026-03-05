//! LLVM wrapper and initialization

use inkwell::targets::{Target, TargetTriple};

pub fn init_llvm() {
    Target::initialize_native(&Default::default()).expect("Failed to initialize LLVM native target");
}

pub fn get_default_triple() -> TargetTriple {
    TargetTriple::create_default()
}
