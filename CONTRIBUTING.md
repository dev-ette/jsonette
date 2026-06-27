# Contributing to jsonette

Thank you for your interest in contributing to `jsonette`! 

`jsonette` is a small, deliberately scoped project that values **correctness, stability, extreme performance, and a clean native user experience over feature breadth**. Before you begin working on a contribution, please read this guide and review the active Architecture Decision Records (ADRs) under `/docs/architecture/decisions/`.

---

## 🧭 The Golden Rules

Every contribution must adhere to these three core principles:

### 1. Respect the Engine / Shell Boundary (ADR-0003)
All business logic—parsing, tree model representation, formatting, JSONPath evaluation, autocomplete schema inference, and error diagnostics—lives entirely in the **Rust core engine**. 
- Shell applications (SwiftUI, Tauri, etc.) **only render**. 
- **Do not** add JSON parsing, regex queries, or schema validation inside a shell. If a feature involves data processing, it belongs in `/engine`.

### 2. Privacy First
`jsonette` is 100% local. 
- **No network calls** are allowed.
- **No telemetry, logging services, or crash reporters** that connect to external servers may be added.
- Pull requests that introduce network connectivity of any kind will be rejected.

### 3. Strict Performance Budgets
Performance is a core feature. We enforce limits on cold start times, idle RAM consumption, and input latency.
- Avoid pulling in heavy crates with large transitive dependency trees in `/engine/Cargo.toml`.
- Any PR that regresses typing latency in the editor or parsing speeds on files larger than 10MB will be flagged for optimization.

---

## 🛠️ Local Development Setup

### 1. Working on the Rust Engine (`/engine`)
The engine is a standard Rust library crate. Ensure you have the version specified in `rust-toolchain.toml` installed.

```bash
# Build the engine in debug mode
cargo build

# Run the unit and property tests
cargo test

# Run lints and formatting checks (CI-enforced)
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

### 2. Working on the macOS Shell (`/macos`)
- Open the Xcode workspace under `/macos`.
- The macOS project is configured to link the Rust engine via UniFFI. Setting up and compiling the project automatically runs the UniFFI bridge compiler.

### 3. Documentation
To generate and view the combined API documentation portal locally:
```bash
./docs/build-docs.sh
open docs/index.html
```

---

## 📋 Pull Request Process

We value focused, incremental progress:

1. **One Focus Per PR**: Avoid bundling multiple features or unrelated refactors into a single pull request. Keep your changes targeted.
2. **Issue Association**: Ensure there is an open issue discussing the feature or bug, and link it to your PR. For substantial design changes, discuss them first on an issue before drafting code.
3. **Robust Testing**:
   - Engine logic changes require unit test coverage.
   - For parsing and formatting roundtrips, prefer writing property-based tests (using `proptest`).
4. **Code Quality**:
   - Keep public FFI API surfaces stable and well-documented.
   - Document any new module, struct, or public function.
   - Ensure `cargo clippy` passes without warnings.

---

## 📜 Code of Conduct

Be constructive, respectful, and collaborative. Please review and adhere to the [Code of Conduct](CODE_OF_CONDUCT.MD).
