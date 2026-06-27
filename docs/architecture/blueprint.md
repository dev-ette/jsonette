# System Architecture Blueprint

Our system modeling is designed in StarUML (`architecture.mdj`) and exported to interactive HTML documentation.

---

## 🔗 Interactive Blueprints

The exported StarUML model contains our structural classes, data flows, and FFI boundaries:

<a href="../html-docs/index.html" class="md-button md-button--primary" target="_blank" style="display: inline-block; margin: 1.5rem 0; padding: 0.6rem 1.2rem; font-weight: bold; background-color: var(--md-primary-color); color: white; border-radius: 4px; text-decoration: none;">Open Interactive Model Viewer ↗</a>

*Note: The model viewer is designed with a multi-pane frameset layout and is best viewed on larger screens.*

---

## 🏗️ Model Layout

The StarUML blueprints cover:

1. **FFI Bridge Design**: Class specifications showing how the SwiftUI client interacts with the Rust engine via UniFFI.
2. **Parser Component**: The data flow diagram of incremental parsing and diagnostic line/column translations.
3. **Query Engine**: Component structures displaying how JSONPath expressions are evaluated against the document tree model.
