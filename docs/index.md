# jsonette Documentation

Welcome to the `jsonette` documentation portal!

`jsonette` is a native, fast, lightweight JSON editor, viewer, and query tool. It consists of a high-performance shared core library written in Rust and native shell applications per operating system (starting with macOS SwiftUI).

---

## 🧭 Documentation Map

This site contains all the technical details, architecture records, and code references for the project:

### 📐 [Architecture Decisions (ADRs)](architecture/decisions/0000-register.md)

Our architectural decision records (ADRs) log the choices made regarding languages, design boundaries, libraries, licensing, and naming.

### 🖼️ [System Architecture Blueprint (StarUML)](architecture/blueprint.md)

Explore our interactive, detailed system blueprint exported from our StarUML modeling tool. It outlines structural models, data structures, and component FFI boundaries.

### 🦀 [Rust Engine API Reference (cargo doc)](engine-api.md)

Access the low-level API documentation generated from the Rust core code, detailing structs, tolerant parsers, FFI entry points, and formatters.

---

## 🛠️ Project Scope and Design Rules

We enforce a strict separation of concerns to support multi-platform shells:

1. **Engine Owns**: Parsing, document AST modeling, JSONPath query evaluation, key completion, formatting, and diagnostics computing.
2. **Shell Owns**: Display rendering, keyboard bindings, and native text input handling.

For instructions on building the project locally, see the [Root README](https://github.com/aharbii/jsonette).
