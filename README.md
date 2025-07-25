<div align="center">
  <img src="https://capsule-render.vercel.app/api?type=waving&height=220&text=Git%20Time%20Traveler&color=0:2A2A2A,100:1A1A1A&fontColor=E0E0E0&fontSize=70&animation=fadeIn&fontAlignY=40&desc=Travel%20back%20in%20time%20on%20your%20GitHub%20profile.&descAlignY=65&descSize=18" alt="Git Time Traveler Header">
</div>

<div align="center">
  <a href="https://git.io/typing-svg">
    <img src="https://readme-typing-svg.herokuapp.com?font=Space+Mono&weight=600&duration=4000&pause=1000&color=909090&width=480&lines=Rust-Powered+%26+Cross-Platform;Interactive+%26+Beautiful+CLI;Backdate+Git+Commits+with+Ease" alt="Typing Animation"/>
  </a>
</div>

<div align="center">
  <p>
    <a href="https://github.com/chama-x/Git-Timetraveler/actions/workflows/release.yml">
      <img src="https://img.shields.io/github/actions/workflow/status/chama-x/Git-Timetraveler/release.yml?branch=main&style=flat-square&label=Release&logo=github&color=2A2A2A&logoColor=E0E0E0" alt="Release Status Badge">
    </a>
    <a href="https://opensource.org/licenses/MIT">
      <img src="https://img.shields.io/badge/License-MIT-informational?style=flat-square&color=383838&logoColor=E0E0E0" alt="License Badge">
    </a>
  </p>
</div>

<div align="center">
  <p>
    <img src="https://img.shields.io/badge/Rust-Powered-000000?style=flat-square&logo=rust&logoColor=E0E0E0&color=2A2A2A" alt="Rust Badge">
    <img src="https://img.shields.io/badge/Windows-Supported-0078D6?style=flat-square&logo=windows&logoColor=E0E0E0&color=2A2A2A" alt="Windows Badge">
    <img src="https://img.shields.io/badge/macOS-Supported-000000?style=flat-square&logo=apple&logoColor=E0E0E0&color=2A2A2A" alt="macOS Badge">
    <img src="https://img.shields.io/badge/Linux-Supported-FCC624?style=flat-square&logo=linux&logoColor=E0E0E0&color=2A2A2A" alt="Linux Badge">
    <img src="https://img.shields.io/badge/npm-Installer-CB3837?style=flat-square&logo=npm&logoColor=E0E0E0&color=383838" alt="NPM Installer Badge">
  </p>
  
  <p>
    <a href="https://www.npmjs.com/package/git-timetraveler">
      <img src="https://img.shields.io/npm/v/git-timetraveler?style=for-the-badge&logo=npm" alt="npm version">
    </a>
  </p>
</div>

A modern, cross-platform rewrite of the original [1990-script](https://github.com/antfu/1990-script) in **Rust**, designed to create GitHub repositories with backdated commits. Enhance your contribution graph by showing activity in earlier years.

---

### Core Features

* **Rust-powered**: Fast, reliable, and memory-safe performance.
* **Cross-platform**: Single binary works on macOS, Windows, and Linux.
* **Interactive CLI**: User-friendly prompts guide you through the process.
* **Secure**: Uses GitHub personal access tokens for auth and performs all operations locally.
* **Zero Dependencies**: No runtime requirements needed for the executable.

---

### Installation & Usage

> **Note:** The npm package is published on [npmjs.com](https://www.npmjs.com/package/git-timetraveler). GitHub Packages and npmjs.com are separate registries; only packages published to GitHub Packages appear in the GitHub "Packages" tab.

#### Recommended: `npx`
The easiest way to run the tool without a manual installation.

```bash
npx git-timetraveler --year 1990
```

You can also specify a custom repository name using the `--repo` flag:

```bash
npx git-timetraveler --year 1990 --repo Git-Timetraveler
```

To create commits for a range of years, use the `--years` flag:

```bash
npx git-timetraveler --years 1990-2001 --repo Git-Timetraveler
```

If `--repo` is not provided, the repository name defaults to the year.

- `--year` and `--years` are mutually exclusive. Use only one at a time.

#### Manual Installation

Download the appropriate binary for your system from the [**Releases Page**](https://github.com/chama-x/Git-Timetraveler/releases).

#### Run Interactively

Simply execute the command to be guided by interactive prompts.

```bash
git-timetraveler
```

*You will be asked for your GitHub username, a personal access token, and the desired date.*

-----

### GitHub Setup

1.  **Create a Repository**: On GitHub, create a new, empty repository. The name should ideally match the year (e.g., `1990`).
2.  **Generate a Token**: Go to `Settings` → `Developer settings` → `Personal access tokens` → `Tokens (classic)`.
      * Click "Generate new token (classic)".
      * Grant the `repo` scope (Full control of private repositories).
      * Copy the generated token to use in the tool.

-----

### How It Works

The tool automates the `git` process for creating a commit with a specific, historical date.

1.  **Clones** your newly created empty repository.
2.  **Creates** a `README.md` file within the local clone.
3.  **Commits** the file using a custom author and committer date based on your input.
      * This is done by setting the `GIT_AUTHOR_DATE` and `GIT_COMMITTER_DATE` environment variables.
4.  **Pushes** the backdated commit to your GitHub repository.
5.  **Cleans up** the local directory.

GitHub's contribution graph renders commits based on the *author date*, which is how the historical square appears on your profile.

<div align="center">
  <img src="https://capsule-render.vercel.app/api?type=waving&height=120&reversal=true&color=0:1A1A1A,50:2A2A2A,100:383838&animation=fadeIn&section=footer" alt="Minimal Footer">
</div>

## Usage

### Interactive Menu (Recommended for Local Terminals)

Run the CLI in a supported terminal (e.g., bash, zsh, Terminal.app, iTerm2) to use the beginner-friendly interactive menu:

```sh
npx git-timetraveler
```

You will be guided through all required options with prompts and validation.

---

### Non-Interactive Mode (For npx, CI, or Scripts)

If you are running in a non-interactive environment (e.g., npx, CI, or a script), use the `--no-menu` flag and provide all required arguments:

```sh
npx git-timetraveler --no-menu --username <user> --token <token> --repo <repo> --year <year>
```

Or for a range of years:

```sh
npx git-timetraveler --no-menu --username <user> --token <token> --repo <repo> --years 2000-2005
```

**Required arguments for --no-menu:**
- `--username` (GitHub username)
- `--token` (GitHub personal access token)
- `--repo` (Repository name)
- `--year` or `--years` (Year or range)

Other options:
- `--branch` (target branch, default: main)
- `--month`, `--day`, `--hour`, `--force`, etc.

If any required argument is missing, the CLI will print an error and exit.

---

**Note:**
- The interactive menu requires a real TTY. If you see a panic or error about `min <= max` or TTY, use `--no-menu` mode.
- For automation, always use `--no-menu` and supply all arguments.
