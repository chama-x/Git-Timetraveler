# git-timetraveler

> 🚀 Travel back in time on your GitHub profile!

This npm package provides easy installation of the [Git Time Traveler](https://github.com/chama-x/Git-Timetraveler) CLI tool.

## Quick Start

```bash
# Run directly with npx (recommended)
npx git-timetraveler --year 1990

# Or install globally
npm install -g git-timetraveler
git-timetraveler --year 1990
```

## What it does

Creates GitHub repositories with backdated commits to show early years in your contribution graph. This is a modern, cross-platform rewrite of the original [1990-script](https://github.com/antfu/1990-script) in Rust.

## Features

- 🦀 **Rust-powered**: Fast, reliable, and memory-safe
- 🌍 **Cross-platform**: Works on macOS, Windows, and Linux
- 🎨 **Beautiful CLI**: Interactive prompts with progress bars
- 📅 **Flexible dates**: Customize year, month, day, and hour
- 🔒 **Secure**: Uses GitHub personal access tokens

## Usage

```bash
# Interactive mode
npx git-timetraveler

# Custom date and time
npx git-timetraveler --year 1985 --month 10 --day 26 --hour 9

# Non-interactive mode
npx git-timetraveler --username johndoe --token ghp_xxxx --year 1990 --yes

# Get help
npx git-timetraveler --help
```

## Installation

This package automatically downloads the appropriate binary for your platform during installation:

- **macOS**: Intel and Apple Silicon support
- **Windows**: x64 support  
- **Linux**: x64 support

No additional dependencies required!

## Requirements

1. **Create a repository** on GitHub with the year as the name (e.g., `1990`)
2. **Generate a personal access token**:
   - Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
   - Click "Generate new token (classic)"
   - Select scopes: `repo` (Full control of private repositories)
   - Copy the generated token

## Repository

- **Source**: [chama-x/Git-Timetraveler](https://github.com/chama-x/Git-Timetraveler)
- **Issues**: [Report bugs](https://github.com/chama-x/Git-Timetraveler/issues)
- **Releases**: [Download binaries](https://github.com/chama-x/Git-Timetraveler/releases)

## License

MIT © [chama-x](https://github.com/chama-x) 