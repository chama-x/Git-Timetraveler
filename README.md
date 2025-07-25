# 🚀 Git Time Traveler

[![Release](https://github.com/chama-x/Git-Timetraveler/actions/workflows/release.yml/badge.svg)](https://github.com/chama-x/Git-Timetraveler/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **Travel back in time on your GitHub profile!** 

Create GitHub repositories with backdated commits to show early years in your contribution graph. This is a modern, cross-platform rewrite of the original [1990-script](https://github.com/antfu/1990-script) in Rust with enhanced features and better UX.

![Demo](https://user-images.githubusercontent.com/placeholder/demo.gif)

## ✨ Features

- 🦀 **Rust-powered**: Fast, reliable, and memory-safe
- 🌍 **Cross-platform**: Works on macOS, Windows, and Linux
- 🎨 **Beautiful CLI**: Interactive prompts with progress bars and colors
- 📅 **Flexible dates**: Customize year, month, day, and hour
- 🔒 **Secure**: Uses GitHub personal access tokens
- 📦 **Easy installation**: Available via npm or direct download
- 🚀 **Zero dependencies**: Single binary with no runtime requirements

## 🚀 Quick Start

### Option 1: npx (Recommended)

The easiest way to get started:

```bash
npx @chamax/git-timetraveler --year 1990
```

### Option 2: Direct Installation

Download the binary for your platform from the [releases page](https://github.com/chama-x/Git-Timetraveler/releases).

### Option 3: Build from Source

```bash
git clone https://github.com/chama-x/Git-Timetraveler.git
cd Git-Timetraveler
cargo build --release
./target/release/git-timetraveler --help
```

## 📖 Usage

### Interactive Mode

Simply run the command and follow the prompts:

```bash
git-timetraveler
```

### Command Line Flags

```bash
git-timetraveler [OPTIONS]

Options:
  -y, --year <YEAR>          Year to travel back to (e.g., 1990) [default: 1990]
  -u, --username <USERNAME>  GitHub username
  -t, --token <TOKEN>        GitHub personal access token
  -m, --month <MONTH>        Month (1-12) [default: 1]
  -d, --day <DAY>            Day (1-31) [default: 1]
      --hour <HOUR>          Hour (0-23) [default: 18]
  -y, --yes                  Skip confirmation prompts
  -h, --help                 Print help
  -V, --version              Print version
```

### Examples

```bash
# Basic usage - travel to 1990
git-timetraveler --year 1990

# Custom date and time
git-timetraveler --year 1985 --month 10 --day 26 --hour 9

# Non-interactive mode
git-timetraveler --username johndoe --token ghp_xxxx --year 1990 --yes

# Travel to your birth year
git-timetraveler --year 1995 --month 3 --day 15
```

## 🔑 GitHub Setup

1. **Create a repository** on GitHub with the year as the name (e.g., `1990`)
2. **Generate a personal access token**:
   - Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
   - Click "Generate new token (classic)"
   - Select scopes: `repo` (Full control of private repositories)
   - Copy the generated token

3. **Run the tool** with your credentials

## 🛠️ How It Works

This tool leverages Git's ability to set custom author and committer dates for commits:

1. **Creates a temporary repository** in your specified year directory
2. **Initializes Git** and creates a README file
3. **Makes a commit** with the backdated timestamp (e.g., `1990-01-01T18:00:00`)
4. **Pushes to GitHub** using your personal access token
5. **Cleans up** the temporary directory

GitHub recognizes the commit timestamp and displays it in your contribution graph for that historical date.

## 🎯 Why Use This?

- **Portfolio enhancement**: Show long-term commitment to coding
- **Profile aesthetics**: Fill gaps in your contribution graph
- **Conversation starter**: Unique profile feature
- **Historical projects**: Backdate the start of long-running projects

## 🔒 Security & Privacy

- Your GitHub token is never stored or logged
- All operations happen locally except for the final push
- The tool only creates public repositories with minimal content
- No personal data is collected or transmitted

## 🚧 Development

### Prerequisites

- Rust 1.70+ 
- Git
- GitHub account

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/git-timetraveler.git
cd git-timetraveler

# Build in debug mode
cargo build

# Build optimized release
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- --help
```

### Project Structure

```
git-timetraveler/
├── src/
│   └── main.rs          # Main CLI application
├── npm/                 # npm wrapper package
│   ├── package.json
│   └── install.js       # Binary download script
├── .github/workflows/
│   └── release.yml      # CI/CD pipeline
├── Cargo.toml           # Rust dependencies
└── README.md
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Original idea from [@antfu](https://github.com/antfu)'s [1990-script](https://github.com/antfu/1990-script)
- Built with ❤️ using [Rust](https://rust-lang.org/)
- CLI powered by [clap](https://github.com/clap-rs/clap)
- Git operations via [git2](https://github.com/rust-lang/git2-rs)

## ⚠️ Disclaimer

This tool is for educational and portfolio enhancement purposes. Use responsibly and in accordance with GitHub's Terms of Service. The created repositories will be public and visible to others.

---

**Happy time traveling! 🕰️✨**
