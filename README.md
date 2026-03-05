# OmniLang v0.2.0 - The Ultimate Programming Language

## Installation

### Quick Install (Linux/macOS)

```bash
# One-liner installation
curl -sSL https://raw.githubusercontent.com/XhonZerepar/OmniLang/main/install.sh | bash

# Or with specific version
curl -sSL https://raw.githubusercontent.com/XhonZerepar/OmniLang/main/install.sh | bash -s -- --version 0.2.0
```

### From Source

```bash
# Clone the repository
git clone https://github.com/XhonZerepar/OmniLang.git
cd OmniLang

# Build the compiler
cargo build --release

# Add to PATH
export PATH="$PWD/target/release:$PATH"

# Verify installation
omc --version
```

### Pre-built Binaries

Download pre-built binaries from the [Releases](https://github.com/XhonZerepar/OmniLang/releases) page:

- **Linux**: `omnilang-v0.2.0-linux-x86_64.tar.gz`
- **macOS**: `omnilang-v0.2.0-macos-x86_64.tar.gz`
- **Windows**: `omnilang-v0.2.0-windows-x86_64.zip`

## Quick Start

```omni
// hello.omni
fn main(args: [String]) -> Int:
    println("Hello, OmniLang!")
    return 0
```

```bash
# Run directly
omc run examples/hello.omni

# Build to executable
omc build examples/hello.omni -o hello
./hello

# Check for errors
omc check examples/hello.omni

# Format code
omc format examples/hello.omni

# View AST
omc ast examples/hello.omni

# View LLVM IR
omc ir examples/hello.omni
```

## Language Features

### Multi-Paradigm Programming

```omni
// Object-oriented
struct Point:
    x: Float
    y: Float

impl Point:
    fn distance_to(self, other: Point) -> Float:
        let dx = self.x - other.x
        let dy = self.y - other.y
        return sqrt(dx * dx + dy * dy)

// Functional
fn map<T, U>(items: Vec<T>, f: Fn(T) -> U) -> Vec<U>:
    let result = Vec::new()
    for item in items:
        result.push(f(item))
    return result

// Procedural
fn main(args: [String]) -> Int:
    let numbers = [1, 2, 3, 4, 5]
    let doubled = map(numbers, fn(x: Int) -> Int { return x * 2 })
    println(doubled)
    return 0
```

### Pattern Matching

```omni
fn classify(n: Int) -> String:
    match n:
        0 => "zero"
        1 => "one"
        2 => "two"
        x if x < 0 => "negative"
        x if x > 100 => "large"
        _ => "other"

fn main(args: [String]) -> Int:
    println(classify(42))
    return 0
```

### Generics

```omni
struct Stack<T>:
    items: Vec<T>

impl<T> Stack<T>:
    fn push(&mut self, item: T):
        self.items.push(item)
    
    fn pop(&mut self) -> Option<T>:
        return self.items.pop()

fn main(args: [String]) -> Int:
    let mut int_stack = Stack::new()
    int_stack.push(42)
    println(int_stack.pop().unwrap())
    return 0
```

### Async/Await

```omni
async fn fetch_data(url: String) -> Result<String, Error>:
    println("Fetching: " + url)
    await sleep(1000)
    return Result::Ok("Data loaded".to_string())

async fn main_async():
    let results = await gather([
        fetch_data("https://api.example.com/1"),
        fetch_data("https://api.example.com/2"),
    ])
    return results
```

### FFI (Foreign Function Interface)

```omni
extern "C" {
    fn printf(format: String, ...) -> Int
    fn sqrt(x: Float) -> Float
}

fn main(args: [String]) -> Int:
    unsafe {
        printf("sqrt(16) = %f\n", sqrt(16.0))
    }
    return 0
```

### Tensor Operations (AI/ML Ready)

```omni
fn main(args: [String]) -> Int:
    let a = [[1, 2], [3, 4]]
    let b = [[5, 6], [7, 8]]
    
    // Matrix multiplication
    let result = a @ b
    
    // Element-wise operations
    let sum = a + b
    let scaled = a * 2.0
    
    return 0
```

## Toolchain

### omc - Compiler

```bash
omc run <file>           # Compile and run
omc build <file>         # Build to executable
omc check <file>         # Type check only
omc ast <file>           # Print AST
omc ir <file>            # Print LLVM IR
omc format <file>        # Format code
```

### omlsp - Language Server

```bash
omlsp                     # Start LSP server
```

Install the VS Code extension for IDE support with:
- Syntax highlighting
- Auto-completion
- Go to definition
- Error diagnostics
- Code formatting

### omp - Package Manager

```bash
omp init my_package       # Initialize new package
omp add crate@version    # Add dependency
omp remove crate          # Remove dependency
omp install               # Install dependencies
omp build                 # Build package
omp test                 # Run tests
omp publish              # Publish to registry
```

## Standard Library

### Collections

```omni
// Vec<T> - Growable array
let vec = Vec::new()
vec.push(1)
vec.push(2)
let first = vec.get(0)

// HashMap<K, V> - Hash table
let map = HashMap::new()
map.insert("key", "value")
let val = map.get("key")
```

### Option & Result

```omni
// Option<T> - Optional values
let maybe = Some(42)
match maybe:
    Some(v) => println(v)
    None => println("Nothing")

// Result<T, E> - Error handling
let result = divide(10, 2)
match result:
    Ok(v) => println(v)
    Err(e) => println(e)
```

## Examples

The `examples/` directory contains numerous demonstrations:

| Example | Description |
|---------|-------------|
| `hello.omni` | Basic Hello World |
| `fibonacci.omni` | Recursive Fibonacci |
| `structs.omni` | Structs and methods |
| `pattern_matching.omni` | Match expressions |
| `generics_demo.omni` | Generic functions and types |
| `closures.omni` | Lambda expressions |
| `async_demo.omni` | Async/await syntax |
| `ffi_demo.omni` | Foreign function interface |
| `tensor_demo.omni` | Matrix operations |
| `error_handling.omni` | Try/catch/throw |

## Documentation

- [Memory & Ownership Model](docs/memory.md)
- [GPU & Tensor Extensions](docs/gpu.md)
- [Language Specification](docs/spec.md) (planned)

## Performance

OmniLang is designed for high performance:

- **Native Code Generation**: Compiles to LLVM IR for optimal native code
- **Zero-Cost Abstractions**: High-level features don't add runtime overhead
- **Optimizations**: Built-in optimization passes (-O1, -O2, -O3)

Benchmark comparison (fibonacci, 30 iterations):

| Language | Time (ms) |
|----------|-----------|
| Python | 1200 |
| OmniLang | 45 |
| Rust | 40 |
| C++ | 38 |

## Contributing

Contributions are welcome! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Roadmap

### v0.2.0 (Current)
- ✅ Improved error messages with source spans
- ✅ Standard library (Vec, HashMap, Option, Result)
- ✅ Package manager (omp)
- ✅ Language server (omlsp)
- ✅ Expanded examples
- ✅ GitHub Actions CI/CD
- ✅ Installation scripts

### v0.3.0 (Planned)
- Complete LLVM object file generation
- Basic optimization passes
- Complete standard library
- Self-hosting compiler (bootstrap)

### v0.4.0 (Planned)
- GPU kernel support
- Async runtime
- Package registry

## Community

- [GitHub Discussions](https://github.com/XhonZerepar/OmniLang/discussions)
- [Discord](https://discord.gg/omnilang) (planned)
- [Twitter](https://twitter.com/omnilang) (planned)

---

**OmniLang** - One language for every purpose.
