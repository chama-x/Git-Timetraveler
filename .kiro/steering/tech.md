# Technology Stack

## Core Technologies
- **Language**: Rust (2021 edition)
- **Build System**: Cargo
- **Target Platforms**: macOS, Windows, Linux (x64, ARM64)

## Key Dependencies
- **CLI Framework**: `clap` v4.4 with derive features for argument parsing
- **Interactive UI**: `dialoguer` v0.11 for prompts and menus
- **HTTP Client**: `reqwest` v0.11 with JSON and rustls-tls features
- **Async Runtime**: `tokio` v1.0 with full features
- **Git Operations**: `git2` v0.18 for Git repository manipulation
- **Date/Time**: `chrono` v0.4 with serde features
- **Error Handling**: `anyhow` v1.0 for error context
- **Progress Indicators**: `indicatif` v0.17
- **Terminal Colors**: `colored` v2.0
- **File Operations**: `tempfile` v3.8 for temporary directories

## Architecture Patterns
- **Modular Design**: Separate modules for git operations, GitHub client, interactive prompts, session management
- **Async/Await**: Tokio-based async operations for HTTP requests and I/O
- **Error Propagation**: Consistent use of `anyhow::Result` with context
- **Configuration Structs**: Strongly typed configuration objects with validation
- **Trait-based Progress**: `ProgressCallback` trait for flexible progress reporting

## Common Commands

### Development
```bash
# Build the project
cargo build

# Run with arguments
cargo run -- --year 1990 --username myuser

# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy linter
cargo clippy
```

### Release
```bash
# Build optimized release
cargo build --release

# The binary will be in target/release/git-timetraveler
```

### NPM Distribution
- Uses `install.js` script to download appropriate binary for platform
- Binaries stored in `bin/` directory
- Package published to npmjs.org registry

## Code Style Guidelines
- Use `anyhow::Result` for error handling with context
- Prefer `async/await` for I/O operations
- Use structured logging and progress reporting
- Validate inputs early with descriptive error messages
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)