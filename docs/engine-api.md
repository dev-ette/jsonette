# Rust Engine API Reference

The `jsonette` core library documentation is generated directly from our Rust source code docstrings.

---

## 🔗 Crate Documentation

The generated API reference lists public modules, parser structs, diagnostics translation helpers, and formatters:

<a href="../engine-docs/jsonette/index.html" class="md-button md-button--primary" target="_blank" style="display: inline-block; margin: 1.5rem 0; padding: 0.6rem 1.2rem; font-weight: bold; background-color: var(--md-primary-color); color: white; border-radius: 4px; text-decoration: none;">Open Rust API Reference ↗</a>

---

## 📦 Key API Components

The rustdocs cover:

- **[`Parser`](../engine-docs/jsonette/parser/index.html)**: Handles parsing JSON text and generating tolerant syntax trees.
- **[`Formatter`](../engine-docs/jsonette/formatter/index.html)**: Manages pretty printing, indentation, and minification.
- **[`QueryEngine`](../engine-docs/jsonette/query/index.html)**: Evaluates RFC 9535 JSONPath expressions.

---

## 🌐 FFI Binding Compatibility Matrix

The core Rust engine is wrapped with UniFFI. Compatibility is verified on every PR commit using compiled and executed smoke-test programs:

| Language    | Bindings Engine  | Verification Method              | Status    | CI Status                                                                                                                                                            |
| :---------- | :--------------- | :------------------------------- | :-------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Swift**   | UniFFI (Native)  | Compiled & Executed via `swiftc` | Supported | [![CI Status](https://github.com/dev-ette/jsonette/actions/workflows/engine-ci.yml/badge.svg)](https://github.com/dev-ette/jsonette/actions/workflows/engine-ci.yml) |
| **Python**  | UniFFI (Native)  | Executed via `python3`           | Supported | [![CI Status](https://github.com/dev-ette/jsonette/actions/workflows/engine-ci.yml/badge.svg)](https://github.com/dev-ette/jsonette/actions/workflows/engine-ci.yml) |
| **C / C++** | UniFFI (Header)  | Compiled & Executed via `clang`  | Supported | [![CI Status](https://github.com/dev-ette/jsonette/actions/workflows/engine-ci.yml/badge.svg)](https://github.com/dev-ette/jsonette/actions/workflows/engine-ci.yml) |
| **Kotlin**  | UniFFI (Native)  | Generated via `uniffi-bindgen`   | Supported | [![CI Status](https://github.com/dev-ette/jsonette/actions/workflows/engine-ci.yml/badge.svg)](https://github.com/dev-ette/jsonette/actions/workflows/engine-ci.yml) |
| **Java**    | UniFFI (JVM/JNA) | Shared via Kotlin JNA Bindings   | Supported | [![CI Status](https://github.com/dev-ette/jsonette/actions/workflows/engine-ci.yml/badge.svg)](https://github.com/dev-ette/jsonette/actions/workflows/engine-ci.yml) |
| **JS / TS** | WebAssembly      | Planned (Milestone 4)            | Roadmap   | Planned                                                                                                                                                              |

Detailed instructions on generating bindings, compilation flags, and linking are available in the [UniFFI Swift Bridge Reference](https://github.com/dev-ette/jsonette/blob/main/engine/BRIDGE.md).
