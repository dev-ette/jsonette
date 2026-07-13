# jsonette

[![Crates.io](https://img.shields.io/crates/v/jsonette-core.svg)](https://crates.io/crates/jsonette-core)
[![Documentation](https://docs.rs/jsonette/badge.svg)](https://docs.rs/jsonette)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT)

A lightweight, zero-dependency, UI-agnostic JSON parse, format, query, and diagnostics engine written in Rust. It serves as the shared core for the `jsonette` native editors and viewers.

---

## Features

- **Tolerant & Incremental Parsing**: Built to handle malformed or partially written JSON documents in real-time editor views.
- **Tree Model Inference**: Builds a lightweight virtual tree representing the JSON hierarchy, ideal for lazy/virtualized outline views.
- **Standardized Querying**: Fully supports RFC 9535 JSONPath queries using `serde_json_path`.
- **Intelligent Autocomplete**: Infers available keys, types, and completions at specific cursor offsets or JSONPath strings.
- **Diagnostics Translation**: Maps precise byte-offsets to line/column dimensions for UI error highlighting.
- **FFI-Ready**: Engineered with a clean, flat public API compatible with UniFFI for binding to native shells (e.g., Swift/SwiftUI, Tauri, Kotlin).

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
jsonette = "0.1.0"
```

---

## Quick Start Usage

### 1. Parsing & Querying JSON

```rust
use jsonette::{Parser, QueryEngine};

fn main() {
    let json_data = r#"
    {
        "store": {
            "book": [
                { "category": "reference", "price": 8.95 },
                { "category": "fiction", "price": 12.99 }
            ]
        }
    }
    "#;

    // Parse the JSON document
    let doc = Parser::parse(json_data).expect("Valid JSON");

    // Execute a JSONPath query
    let results = QueryEngine::query(&doc, "$.store.book[*].price")
        .expect("Valid query syntax");

    println!("Matched prices: {:?}", results);
}
```

### 2. Formatting & Pretty Printing

```rust
use jsonette::Formatter;

fn main() {
    let minified = r#"{"name":"jsonette","version":"0.1.0"}"#;

    // Format JSON with standard 4-space indentation
    let formatted = Formatter::format(minified, 4)
        .expect("Valid JSON input");

    println!("{}", formatted);
}
```

---

## Architecture and FFI Bindings

The `jsonette` engine is deliberately decoupled from any UI framework. The engine maintains 100% of the parse, query, and configuration state, exposing a thread-safe API. For macOS/iOS, it compiles into a static/dynamic library and binds via **UniFFI** to generate Swift wrappers.

For more details on the desktop project structure, see the [Root README](https://github.com/dev-ette/jsonette).

---

## License

Dual-licensed under either:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/dev-ette/jsonette/blob/main/LICENSE-APACHE))
- MIT license ([LICENSE-MIT](https://github.com/dev-ette/jsonette/blob/main/LICENSE-MIT))

at your option.
