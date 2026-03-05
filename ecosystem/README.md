# OmniLang Ecosystem

A comprehensive ecosystem for the OmniLang programming language, including a unified CLI tool, standard library extensions, testing framework, documentation generator, and project templates.

## Contents

- [Unified CLI](#unified-cli)
- [Standard Library](#standard-library)
- [Testing Framework](#testing-framework)
- [Documentation Generator](#documentation-generator)
- [Project Templates](#project-templates)
- [Installation](#installation)
- [Quick Start](#quick-start)

## Unified CLI

The `omni` command provides a unified interface for all OmniLang development tasks:

```bash
# Initialize a new project
omni init my-project

# Build the project
omni build
omni build --release

# Run the project
omni run
omni run --release -- arg1 arg2

# Run tests
omni test

# Format code
omni fmt
omni fmt src/main.omni

# Generate documentation
omni doc

# Manage dependencies
omni install http
omni install json@1.2.0
omni remove http
omni update

# Create files from templates
omni new struct MyStruct
omni new test test_name
omni new async fetch_data

# Show version and environment
omni version
omni env
```

### Project Configuration (omni.toml)

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2024"
authors = ["Your Name"]
description = "My OmniLang project"

[dependencies]
http = "1.0"
json = "2.0"

[build]
target = "native"
optimization = "default"
```

## Standard Library

The ecosystem includes extended standard library modules:

### File System (std::fs)

```omni
use std::fs

// Read and write files
let content = fs::read_file("data.txt")?
fs::write_file("output.txt", "Hello!")?

// Directory operations
fs::create_dir_all("logs/2024")?
for entry in fs::read_dir(".")?:
    println(entry.name)

// Path manipulation
let ext = fs::extension("file.txt")  // "txt"
let name = fs::stem("file.txt")       // "file"
let dir = fs::dirname("/path/to/file.txt")  // "/path/to"
```

### HTTP (std::http)

```omni
use std::http

// HTTP client
let client = Client::new()
let response = client.get("https://api.example.com/data")?

if response.is_success():
    let body = response.body.unwrap()
    println(body)

// POST request
let resp = client.post("https://api.example.com/create", json_data)?
```

### DateTime (std::time)

```omni
use std::time

// Current time
let now = DateTime::now()
let today = Date::today()

// Formatting
println(now.format("%Y-%m-%d %H:%M:%S"))

// Arithmetic
let tomorrow = today.add_days(1)
let duration = now.diff(other_time)

// Stopwatch
let sw = Stopwatch::start()
// ... do work ...
let elapsed = sw.elapsed()
println("Took " + elapsed.as_millis().to_string() + "ms")
```

### Testing (std::test)

```omni
use std::test

// Basic assertions
test::assert_eq(42, 40 + 2)
test::assert_true(x > 0)
test::assert_not_null(value)

// Test suites
let suite = TestSuite::new("My Tests")
    .add_case("test_one", || test_one())
    .add_case("test_two", || test_two())
    .before_each(|| setup())
    .after_each(|| teardown())

suite.run()

// Benchmarks
let result = Benchmark::new("my benchmark")
    .with_iterations(10000)
    .run(|| do_work())
result.print()
```

### Package Manager (std::pkg)

```omni
use std::pkg

// Create a package
let manifest = Manifest::new("mylib", "1.0.0")
    .with_description("My awesome library")
    .with_author("Your Name")
    .add_dependency("http", ">=1.0")

// Resolve dependencies
let registry = Registry::default()
let resolver = Resolver::new(registry)
let resolved = resolver.resolve(manifest.dependencies)?
```

### Documentation Generator (std::doc)

```omni
use std::doc

let config = DocConfig::new()
    .with_title("My Library")
    .with_description("Documentation for my project")
    .with_output_dir("target/docs")

generate_docs("src", config)?
```

## Testing Framework

The built-in testing framework supports:

- **Unit tests**: Test individual functions and types
- **Integration tests**: Test component interactions
- **Benchmarks**: Performance testing
- **Property-based testing**: Test with generated inputs

```omni
#[test]
fn test_basic():
    assert_eq(1 + 1, 2)

#[bench]
fn bench_sort():
    let data = generate_large_array()
    sort(data)
```

Run tests:

```bash
omni test
```

## Documentation Generator

Generate beautiful HTML documentation from source code:

```bash
# Generate docs
omni doc

# Or use the tool directly
python3 tools/omnidoc.omni src target/docs
```

Documentation comments use Markdown:

```omni
/// # My Function
/// 
/// This function does something useful.
/// 
/// ## Parameters
/// 
/// - `input`: The input value
/// 
/// ## Returns
/// 
/// The processed result
/// 
/// ## Example
/// 
/// ```
/// let result = my_function(42)
/// ```
pub fn my_function(input: Int) -> Int:
    return input * 2
```

## Project Templates

Quickly scaffold new projects:

### Basic Project

```bash
omni init my-project
```

### Web Application

```bash
omni init my-webapp -t web
```

### Library

```bash
omni init my-library -t lib
```

### CLI Application

```bash
omni init my-cli -t cli
```

Templates include:
- Project structure
- Sample code
- Tests
- README
- Build configuration

## Installation

### Quick Install

```bash
# Download the ecosystem
git clone https://github.com/XhonZerepar/OmniLang.git
cd OmniLang

# Make omni CLI executable
chmod +x omni

# Add to PATH
export PATH="$PWD:$PATH"

# Test
omni version
```

### System-wide Installation

```bash
# Install to ~/.local/bin
mkdir -p ~/.local/bin
cp omni ~/.local/bin/

# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.local/bin:$PATH"
```

## Quick Start

1. **Create a new project**:

   ```bash
   omni init hello-world
   cd hello-world
   ```

2. **Write some code**:

   Edit `src/main.omni`:

   ```omni
   fn main(args: [String]) -> Int:
       println("Hello, OmniLang!")
       return 0
   ```

3. **Run it**:

   ```bash
   omni run
   ```

4. **Add dependencies**:

   ```bash
   omni install http
   ```

5. **Build for release**:

   ```bash
   omni build --release
   ```

## Ecosystem Structure

```
omni_ecosystem/
├── omni                  # Unified CLI tool
├── stdlib/
│   └── std/
│       ├── fs.omni       # File system module
│       ├── http.omni     # HTTP client/server
│       ├── time.omni    # Date/time utilities
│       ├── test.omni    # Testing framework
│       └── pkg.omni     # Package management
├── tools/
│   └── omnidoc.omni     # Documentation generator
└── templates/
    ├── basic/            # Basic project template
    ├── web/              # Web app template
    ├── lib/              # Library template
    └── cli/              # CLI app template
```

## Requirements

- OmniLang compiler (omc) v0.2.0 or later
- Python 3.7+ (for CLI tools)
- TOML library for Python

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure code is formatted
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Support

- GitHub Issues: https://github.com/XhonZerepar/OmniLang/issues
- Discussions: https://github.com/XhonZerepar/OmniLang/discussions
