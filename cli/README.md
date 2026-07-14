# Command Line Interface (CLI) Reference

[![Crates.io](https://img.shields.io/crates/v/jsonette.svg)](https://crates.io/crates/jsonette)
[![Docs Portal](https://img.shields.io/badge/docs-portal-blue)](https://dev-ette.github.io/jsonette/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT)

`jsonette` provides a high-performance, standalone Command Line Interface (CLI) built on top of the core Rust engine. It allows formatting, minifying, querying, and managing configuration settings directly in your terminal.

---

## 🚀 Installation

We provide pre-compiled binaries and official packages for all major operating systems.

### Package Managers

| OS / Ecosystem      | Package Manager | Installation Command                                                                          |
| ------------------- | --------------- | --------------------------------------------------------------------------------------------- |
| **macOS / Linux**   | Homebrew        | `brew install jsonette`                                                                       |
| **Ubuntu / Debian** | APT (PPA)       | `sudo add-apt-repository ppa:dev-ette/stable && sudo apt update && sudo apt install jsonette` |
| **Fedora / RHEL**   | DNF (COPR)      | `sudo dnf copr enable aharbi/jsonette && sudo dnf install jsonette`                           |
| **Arch Linux**      | Pacman (AUR)    | `yay -S jsonette`                                                                             |
| **Windows**         | Winget          | `winget install dev-ette.jsonette`                                                            |
| **Rust Developers** | Cargo           | `cargo install jsonette`                                                                      |

---

### Pre-compiled Binaries (GitHub Releases)

If you prefer a portable standalone executable, download the pre-compiled `.tar.gz` or `.zip` files from our [GitHub Releases](https://github.com/dev-ette/jsonette/releases) page. All release assets are accompanied by a `SHA256SUMS.txt` file for cryptographic verification.

```bash
# Example for macOS Apple Silicon
curl -LO https://github.com/dev-ette/jsonette/releases/latest/download/jsonette-macos-arm64.tar.gz
tar -xvf jsonette-macos-arm64.tar.gz
chmod +x jsonette
sudo mv jsonette /usr/local/bin/
```

---

## 🛠️ Commands & Usage

### 1. Formatting & Minifying

Formats (pretty-prints) or minifies a JSON file or standard input.

```bash
# Format a JSON file (outputs to stdout)
jsonette format data.json

# Format standard input via pipeline
cat data.json | jsonette format

# Minify JSON (remove all whitespace)
jsonette format data.json --minify

# Output formatted JSON to a new file
jsonette format data.json --output formatted.json

# Format in-place (updates the file directly)
jsonette format data.json --in-place
```

#### Formatting Option Overrides

You can override your global configuration for a single command run using the following flags:

- `-o, --output <file>`: Write output to a specific file instead of standard output.
- `-i, --in-place`: Edit the input file in-place (mutually exclusive with `--output`).
- `-s, --sort-keys <true|false>`: Sort object keys alphabetically.
- `-n, --indent <count>`: Set the number of spaces/tabs for indentation.
- `--use-tabs <true|false>`: Use tab characters instead of spaces.
- `--line-ending <lf|crlf>`: Force a specific line ending style.
- `--folding-style <expanded|compact>`: Customize folding behavior.

---

### 2. Querying JSON (JSONPath)

Evaluate RFC 9535 JSONPath expressions against a JSON document.

```bash
# Query a file
jsonette query "$.store.book[*].author" books.json

# Query standard input
cat books.json | jsonette query "$.store.book[*].author"
```

---

### 3. Exploring JSON Structure (`explore`)

For unfamiliar JSON structures, `explore` allows you to discover keys or array lengths without needing to know them in advance.

```bash
# View the keys at the root of a JSON file (defaults to '$')
jsonette explore data.json

# View keys of an object at a specific path
jsonette explore "$[0]" data.json

# Filter keys matching a regex and limit the output to 5 items
jsonette explore --regex "^meta_" -n 5 "$[0]" data.json
```

---

### 4. Converting JSON (`convert`)

Convert a JSON document into other serialization formats (YAML, TOML, XML).

```bash
# Convert a JSON file to YAML
jsonette convert -t yaml data.json

# Convert a JSON file to TOML
jsonette convert -t toml data.json

# Convert standard input to XML
cat data.json | jsonette convert -t xml
```

---

### 5. Generating Dummy Data (`generate`)

Generate structured dummy JSON data from a schema file.

```bash
# Generate dummy JSON based on schema
jsonette generate --schema schema.json

# Generate a 10MB dummy JSON object and write to a file
jsonette generate --schema schema.json --size 10MB -o large-dummy.json
```

---

### 6. Global Config Management

Manage your global settings file (`~/.config/jsonette/settings.json` or `%LOCALAPPDATA%\jsonette\settings.json`) directly from the command line.

```bash
# List all active settings in JSON format
jsonette config list

# Get a specific configuration key
jsonette config get format.sort_keys

# Set a configuration key (persists to disk)
jsonette config set format.sort_keys true
jsonette config set format.indent 4
```

---

### 7. Shell Autocompletion (`completions`)

Dynamically generates autocompletion scripts for various shells including `bash`, `zsh`, `fish`, `powershell`, and `elvish`.

```bash
# Generate zsh autocompletions (prints to stdout)
jsonette completions zsh

# Load completions immediately in your current zsh session
source <(jsonette completions zsh)

# Install completions permanently (macOS/Homebrew zsh setup)
jsonette completions zsh > $(brew --prefix)/share/zsh/site-functions/_jsonette
```

---

## ⚠️ Diagnostics & Error Output

When formatting or parsing invalid JSON, `jsonette` outputs detailed diagnostics to `stderr` with line/column coordinates and a compiler-style visual caret pointer:

```text
Error in <stdin>:2:1: EOF while parsing an object at line 2 column 0
  |
  1 | {"a": 1
  |        ^
```

- Exit code `0` is returned on success.
- Exit code `1` is returned on parsing, evaluation, or syntax errors.
