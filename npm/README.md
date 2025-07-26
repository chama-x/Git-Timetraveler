# Git Time Traveler

Create GitHub repositories with backdated commits to show early years in your profile contribution graph.

## Quick Start

```bash
# Interactive mode (recommended)
npx git-timetraveler

# Non-interactive mode
npx git-timetraveler --no-menu --username myuser --token ghp_xxx --repo myrepo --year 1990
```

## Features

- 🚀 **Fast & Lightweight**: Written in Rust for optimal performance
- 🎯 **Cross-platform**: Works on macOS, Windows, and Linux
- 🔒 **Secure**: Uses GitHub personal access tokens
- 📅 **Flexible dates**: Support for single years, ranges, or lists
- 🎨 **Interactive UI**: User-friendly prompts with smart defaults
- 🔍 **Dry run mode**: Preview operations before execution
- 📦 **Zero dependencies**: Self-contained executable

## Installation

### Via NPX (Recommended)

```bash
npx git-timetraveler
```

### Via NPM

```bash
npm install -g git-timetraveler
git-timetraveler
```

## Usage

### Interactive Mode

Simply run the command and follow the prompts:

```bash
npx git-timetraveler
```

### Non-Interactive Mode

For automation or CI/CD:

```bash
npx git-timetraveler --no-menu \
  --username myuser \
  --token ghp_xxxxxxxxxxxx \
  --repo myrepo \
  --year 1990
```

### Year Ranges

```bash
# Single year
npx git-timetraveler --no-menu --username myuser --token ghp_xxx --repo myrepo --year 1990

# Year range
npx git-timetraveler --no-menu --username myuser --token ghp_xxx --repo myrepo --years 1990-1995

# Specific years
npx git-timetraveler --no-menu --username myuser --token ghp_xxx --repo myrepo --years 1990,1992,1994
```

### Advanced Options

```bash
npx git-timetraveler --no-menu \
  --username myuser \
  --token ghp_xxxxxxxxxxxx \
  --repo myrepo \
  --years 1990-1995 \
  --hour 14 \
  --author-name "John Doe" \
  --author-email "john@example.com" \
  --message "Custom commit message for {year}" \
  --private \
  --create-repo
```

## Options

| Option | Description | Example |
|--------|-------------|---------|
| `--year` | Single year to travel to | `--year 1990` |
| `--years` | Range or list of years | `--years 1990-1995` or `--years 1990,1992,1994` |
| `--username` | GitHub username | `--username myuser` |
| `--token` | GitHub personal access token | `--token ghp_xxxxxxxxxxxx` |
| `--repo` | Repository name | `--repo myrepo` |
| `--branch` | Branch to push to | `--branch main` |
| `--author-name` | Custom author name | `--author-name "John Doe"` |
| `--author-email` | Custom author email | `--author-email "john@example.com"` |
| `--message` | Custom commit message | `--message "Time travel to {year}"` |
| `--hour` | Hour for commits (0-23) | `--hour 14` |
| `--create-repo` | Create repository if it doesn't exist | `--create-repo` |
| `--private` | Make repository private | `--private` |
| `--dry-run` | Preview operations without making changes | `--dry-run` |
| `--yes` | Skip confirmation prompts | `--yes` |
| `--no-menu` | Run in non-interactive mode | `--no-menu` |
| `--verbose` | Enable verbose output | `--verbose` |
| `--quiet` | Minimal output | `--quiet` |

## GitHub Token Setup

1. Go to [GitHub Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens)
2. Click "Generate new token (classic)"
3. Select the following scopes:
   - `repo` (Full control of private repositories)
   - `user` (Read user profile data)
4. Copy the generated token
5. Use it with the `--token` flag or enter it when prompted

## Examples

### Basic Usage

```bash
# Interactive mode - just follow the prompts
npx git-timetraveler

# Create commits for 1990
npx git-timetraveler --no-menu --username johndoe --token ghp_xxx --repo my-1990-project --year 1990

# Create commits for multiple years
npx git-timetraveler --no-menu --username johndoe --token ghp_xxx --repo retro-coding --years 1990-1995
```

### Advanced Usage

```bash
# Custom commit author and message
npx git-timetraveler --no-menu \
  --username johndoe \
  --token ghp_xxxxxxxxxxxx \
  --repo vintage-code \
  --years 1990,1995,2000 \
  --author-name "Past Me" \
  --author-email "pastme@example.com" \
  --message "Nostalgic coding session in {year}"

# Create private repository with custom settings
npx git-timetraveler --no-menu \
  --username johndoe \
  --token ghp_xxxxxxxxxxxx \
  --repo secret-retro-project \
  --year 1990 \
  --create-repo \
  --private \
  --description "My secret time travel experiments"
```

### Dry Run Mode

Preview what will happen without making any changes:

```bash
npx git-timetraveler --dry-run --no-menu \
  --username johndoe \
  --token ghp_xxxxxxxxxxxx \
  --repo test-repo \
  --year 1990
```

## Troubleshooting

### Common Issues

**"This CLI requires an interactive terminal (TTY)"**
- Use the `--no-menu` flag for non-interactive environments
- Provide all required arguments via command line flags

**"Authentication failed"**
- Verify your GitHub token is correct and has the required permissions
- Make sure the token hasn't expired

**"Repository not found"**
- Use `--create-repo` to create the repository automatically
- Verify the repository name and username are correct

**"Binary not found"**
- The binary will be downloaded automatically on first use
- If download fails, try reinstalling: `npm install -g git-timetraveler`

### Getting Help

```bash
# Show help
npx git-timetraveler --help

# Verbose output for debugging
npx git-timetraveler --verbose --no-menu [other options]
```

## Platform Support

- **macOS**: Intel (x64) and Apple Silicon (ARM64)
- **Linux**: x64 and ARM64
- **Windows**: x64 and ARM64

## Security

- Tokens are never logged or displayed in plain text
- All operations can be previewed with `--dry-run`
- Confirmation prompts for destructive operations
- Secure token storage using OS keychain (when available)

## Contributing

Visit the [GitHub repository](https://github.com/chama-x/Git-Timetraveler) to:
- Report bugs
- Request features
- Contribute code
- View source code

## License

MIT License - see the [LICENSE](https://github.com/chama-x/Git-Timetraveler/blob/main/LICENSE) file for details.