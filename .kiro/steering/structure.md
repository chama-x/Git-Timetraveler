# Project Structure

## Root Directory
- `Cargo.toml` - Main Rust project configuration and dependencies
- `Cargo.lock` - Dependency lock file
- `README.md` - Project documentation and usage instructions
- `LICENSE` - MIT license file
- `package.json` - NPM package configuration for distribution

## Source Code (`src/`)
- `main.rs` - CLI entry point with argument parsing and interactive menu
- `lib.rs` - Main library exports and core `create_time_traveled_repo` function
- `git_operations.rs` - Git repository operations (clone, commit, push)
- `github_client.rs` - GitHub API client for repository management
- `git_context.rs` - Git context detection and identity management
- `interactive.rs` - Interactive prompts and user input handling
- `session.rs` - Session management and user preferences
- `defaults.rs` - Smart defaults engine and context analysis

## Distribution (`npm/`)
- `package.json` - NPM package metadata
- `install.js` - Platform-specific binary installer script
- `bin/` - Platform-specific binaries for distribution
- `temp/` - Temporary files during build process

## Examples (`examples/`)
- `basic_usage.rs` - Example usage of the library API

## Build Artifacts
- `target/` - Cargo build output (gitignored)
- `binary/` - Compiled binaries for different platforms

## Scripts
- `install.sh` - Installation script for direct binary usage
- `index.sh` - Main execution wrapper script

## Subprojects
- `Git-Timetraveler-verify/` - Verification tool subproject
- `Git-Timetraveler-bulk-verify/` - Bulk verification tool subproject

## Configuration
- `.kiro/` - Kiro IDE configuration and steering rules
- `.github/` - GitHub Actions workflows and templates
- `.gitignore` - Git ignore patterns

## Module Organization Principles
- **Separation of Concerns**: Each module handles a specific domain (Git ops, GitHub API, UI)
- **Library + Binary**: Core logic in `lib.rs`, CLI interface in `main.rs`
- **Error Boundaries**: Each module defines its own error types and contexts
- **Async Boundaries**: Clear separation between sync and async operations
- **Configuration Objects**: Strongly typed config structs passed between modules

## File Naming Conventions
- Use snake_case for Rust source files
- Use kebab-case for script files and directories
- Generated time travel files follow pattern: `timetravel-{year}.md`
- NPM packages follow pattern: `git-timetraveler-{version}.tgz`