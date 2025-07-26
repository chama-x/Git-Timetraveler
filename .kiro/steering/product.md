# Product Overview

Git Time Traveler is a cross-platform CLI tool written in Rust that creates GitHub repositories with backdated commits to enhance contribution graphs by showing activity in earlier years.

## Core Purpose
- Create backdated Git commits for any specified year (1970-2030)
- Push these commits to GitHub repositories to populate contribution graphs
- Provide both interactive and non-interactive modes for different use cases

## Key Features
- **Cross-platform**: Single binary for macOS, Windows, and Linux
- **Interactive CLI**: User-friendly prompts with smart defaults
- **Secure authentication**: Uses GitHub personal access tokens
- **Flexible date selection**: Support for single years or year ranges
- **NPM distribution**: Available via `npx git-timetraveler` for easy access
- **Zero runtime dependencies**: Self-contained executable

## Target Users
- Developers wanting to enhance their GitHub contribution graphs
- Users migrating from the original 1990-script (JavaScript) to a more robust Rust implementation
- Both technical users (CLI arguments) and non-technical users (interactive menu)

## Distribution
- Primary: NPM package for easy `npx` usage
- Secondary: Direct binary downloads from GitHub releases
- Supports both interactive terminal usage and CI/automation scenarios