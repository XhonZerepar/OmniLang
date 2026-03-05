# Contributing to OmniLang

Thank you for your interest in contributing to OmniLang!

## Getting Started

1. **Fork the repository**
2. **Clone your fork**: `git clone https://github.com/YOUR_USERNAME/OmniLang.git`
3. **Create a branch**: `git checkout -b feature/your-feature-name`

## Development Setup

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/XhonZerepar/OmniLang.git
cd OmniLang
cargo build --release

# Run tests
cargo test
```

## Project Structure

```
OmniLang/
├── src/
│   ├── lexer.rs      # Tokenizer
│   ├── parser.rs     # Parser
│   ├── ast.rs       # Abstract syntax tree
│   ├── typecheck.rs # Type checker
│   ├── codegen.rs   # LLVM code generation
│   └── main.rs      # CLI entry point
├── examples/        # Example programs
├── docs/            # Documentation
└── tests/           # Test suite
```

## Coding Standards

### Rust Code

- Follow Rust's standard formatting (`cargo fmt`)
- Run clippy lints (`cargo clippy`)
- Write tests for new features

### OmniLang Code

- Use meaningful variable names
- Add comments for complex logic
- Follow the existing code style

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

## Pull Request Process

1. **Update documentation** for any changed functionality
2. **Add tests** for new features
3. **Ensure all tests pass**: `cargo test`
4. **Format code**: `cargo fmt`
5. **Run lints**: `cargo clippy`
6. **Submit PR** with clear description

## Issue Guidelines

- Use clear, descriptive titles
- Provide reproduction steps
- Include relevant information (OS, Rust version, etc.)

## Communication

- GitHub Discussions: For questions and general discussion
- GitHub Issues: For bug reports and feature requests

## Recognition

Contributors will be recognized in the README and release notes.

---

**Thank you for contributing to OmniLang!**
