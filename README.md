# jsonette

[![Engine CI](https://github.com/dev-ette/jsonette/actions/workflows/engine-ci.yml/badge.svg)](https://github.com/dev-ette/jsonette/actions/workflows/engine-ci.yml)
[![Docs Portal](https://img.shields.io/badge/docs-portal-blue)](https://dev-ette.github.io/jsonette/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-stable-orange)](engine/rust-toolchain.toml)
[![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)](https://github.com/dev-ette/jsonette)

> A focused, fast, native desktop JSON editor, viewer, and query tool. Lightweight by design, 100% local, and open source.

---

## 🎯 Project Philosophy

Unlike bundled "swiss-army-knife" developer utilities, `jsonette` is designed to do three core tasks exceptionally well, focusing on native-tier quality, performance, and user privacy:

1. **Edit**: High-performance text editor experience with syntax highlighting, inline diagnostic error indicators, and instant formatting.
2. **View**: High-speed, virtualized, collapsible outline tree view with click-to-navigate structural exploration.
3. **Query**: Real-time RFC 9535 JSONPath evaluator featuring intelligent key autocomplete and a live results panel.

### Core Guarantees

- **Strict Privacy**: 100% local processing. Zero telemetry, zero external trackers, and zero network calls.
- **Extreme Performance**: Enforced startup and memory budgets (e.g., fast cold start, under 30MB idle RAM).
- **Native Experience**: Standard native controls and performance over bloated multi-gigabyte Electron apps.

---

## 🏛️ System Architecture

To ensure multi-platform capability without duplicating business logic, `jsonette` enforces a strict separation of concerns between its core engine and the native application shells.

```
┌──────────────────────────────────────────────────────────────┐
│                  CORE ENGINE (Rust Crate)                    │
│   Parsers · Tree Model · Formatter · JSONPath · Autocomplete │
└──────────────┬───────────────┬───────────────┬───────────────┘
               │               │               │
        Direct Dependency   UniFFI FFI   Future Web ABI
               │               │               │
        ┌──────┴──────┐ ┌──────┴──────┐ ┌──────┴──────┐
        ▼             ▼ ▼             ▼ ▼             ▼
     CLI App           macOS Shell     Future Shells
     (clap binary)     (SwiftUI)       (Tauri / Web)
```

- **The Engine (`jsonette-core`)**: Owns parsing logic, query computation, autocomplete schema inference, diagnostics generation, settings management, and data transformations.
- **The CLI (`jsonette`)**: A fast, dependency-isolated terminal binary wrapping the engine, providing pipeline formatting and query capabilities.
- **The GUI Shell (SwiftUI / Tauri)**: Focuses purely on native system integrations, input handling, and high-performance virtualized UI rendering.

---

## 🛠️ Technology Stack

| Layer               | Technology               | Rationale                                                                          |
| ------------------- | ------------------------ | ---------------------------------------------------------------------------------- |
| **Core Engine**     | Rust                     | Portable compiled library, native speed, memory safety. Shared via UniFFI wrapper. |
| **CLI App**         | Rust + Clap              | Dependency-isolated binary for shell scripting and pipelines.                      |
| **macOS UI**        | SwiftUI + AppKit         | Deeply integrated native macOS 14+ look-and-feel.                                  |
| **macOS Editor**    | Tree-sitter / AppKit     | Native text view with high-speed syntax coloring.                                  |
| **macOS Tree View** | Virtualized outline view | Handles multi-gigabyte JSON hierarchies lazily without lag.                        |

---

## 📂 Repository Structure

- **/engine**: Core Rust library crate (`jsonette-core`) containing all parsing, formatting, and diagnostics logic.
- **/cli**: Modular command line application crate (`jsonette`) containing clap CLI args, subcommands, and pipeline formatting helpers.
- **/macos**: SwiftUI client application (Xcode project) consuming the engine.
- **/docs**: Unified documentation portal config, architecture ADRs, and decision records.
- **/.github**: CI/CD pipelines (Engine & CLI validation, docs deployment, crates.io publishing).

---

## 🚀 Getting Started & Building

### Prerequisites

- **Rust Toolchain**: Install via [rustup](https://rustup.rs/) (version details in [engine/rust-toolchain.toml](engine/rust-toolchain.toml)).
- **Xcode**: Required for macOS application compilation.

### Building the Workspace & Running Tests

All commands are run from the workspace root:

```bash
# Build the core engine and CLI
cargo build --workspace

# Run all unit, integration, and property tests
cargo test --workspace

# Run quality checks
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

### Developing documentation portal locally

To generate the Rust engine and CLI API documentation locally:

```bash
# Execute the docs script
./docs/build-docs.sh

# Open generated index in browser
open docs/engine-docs/jsonette/index.html
```

---

## 🚢 CI/CD Pipelines

- **PR Validation**: Tests build, clippy, and unit tests.
- **Documentation Deployment**: Pushes to `main` compile Rust API docs, bundle StarUML html-docs, and deploy to [GitHub Pages](https://dev-ette.github.io/jsonette/).
- **Crates.io Release**: Pushing a tag (e.g. `v0.1.0`) triggers a publication of the `jsonette` crate to [crates.io](https://crates.io/crates/jsonette).

---

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) to understand our coding standards, the strict engine/shell separation rule, and performance budgets before opening a pull request.

---

## 📄 License

Dual-licensed under either:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))
