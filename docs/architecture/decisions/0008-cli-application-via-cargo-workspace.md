# 8. CLI Application and Cargo Workspace Configuration

Date: 2026-07-13
## Status
Accepted

---

## Context

To enhance testing, support developer terminal workflows, and provide a path for early packaging/distribution (e.g. via Homebrew, apt, winget), we want to expose the `jsonette` Rust engine as a standalone CLI application.

However, adding CLI-specific concerns (like command-line argument parsers, terminal color styling, and standard input/output handling) directly to the core `engine` library poses risks:
- It bloats the compilation size and dependency footprint for other consumers (such as the native macOS SwiftUI shell via UniFFI).
- It complicates library target options (UniFFI compilation targets `cdylib`/`staticlib` which conflict with direct binary targets).

Additionally, application configuration options should be persistently stored in standard user configuration directories ($HOME/.config/jsonette/settings.json or %LOCALAPPDATA%\jsonette\settings.json) and be queryable or editable via both the GUI and CLI.

---

## Decisions

### 1. Transition to Cargo Workspace
**Decision:** Restructure the Rust project into a Cargo Workspace:
- Keep the `engine` package as a pure, dependency-minimal library that focuses solely on core JSON parsing, formatting, querying, and diagnostics.
- Create a separate `cli` package that depends on the `engine` package and manages CLI-specific dependencies (such as `clap`).

### 2. Standalone CLI Application
**Decision:** Deliver a standalone `jsonette` CLI tool supporting:
- `format` (and minification) on input files or standard input pipelines.
- Single-run option overrides via command-line arguments (using in-memory settings adjustments that do not persist to disk).
- JSONPath query evaluation.
- Compiler-style diagnostic output mapping byte offsets back to lines and columns with caret indicators.

### 3. Unified Global Config Subcommands
**Decision:** Standardize global settings updates via CLI subcommands (`jsonette config list|get|set`) that write directly to the shared platform configuration path, providing git-style setting management.

---

## Consequences

- The UniFFI binding compilation pipeline remains clean and unaffected by CLI dependencies.
- Command-line formatting can be integrated into pre-commit hooks, text editor plugins, and CI/CD pipelines.
- Enhances test automation capability, allowing direct execution of JSON tests and query evaluation from shell scripts.
