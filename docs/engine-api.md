# Rust Engine API Reference

The `jsonette` core library documentation is generated directly from our Rust source code docstrings.

---

## 🔗 Crate Documentation

The generated API reference lists public modules, parser structs, diagnostics translation helpers, and formatters:

<a href="../engine-docs/jsonette/index.html" class="md-button md-button--primary" target="_blank" style="display: inline-block; margin: 1.5rem 0; padding: 0.6rem 1.2rem; font-weight: bold; background-color: var(--md-primary-color); color: white; border-radius: 4px; text-decoration: none;">Open Rust API Reference ↗</a>

---

## 📦 Key API Components

The rustdocs cover:

- **[`Parser`](../engine-docs/jsonette/praser/index.html)**: Handles parsing JSON text and generating tolerant syntax trees.
- **[`Formatter`](../engine-docs/jsonette/formatter/index.html)**: Manages pretty printing, indentation, and minification.
- **[`QueryEngine`](../engine-docs/jsonette/query/index.html)**: Evaluates RFC 9535 JSONPath expressions.
- **FFI Bindings**: Entrypoints compiled into a static library for native consumption via UniFFI.
