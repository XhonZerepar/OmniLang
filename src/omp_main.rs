//! Package Manager Binary Entry Point

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    omnilang::package_manager::run_omp(args);
}
