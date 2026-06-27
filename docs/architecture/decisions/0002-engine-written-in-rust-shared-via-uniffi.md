# 2. Engine written in Rust, shared across shells via UniFFI

Date: 2026-06-27
## Status
Accepted

---

## Context

Under [ADR-0001](0001-native-per-os-macos-first-with-tauri-comparison.md) the engine may be consumed first by SwiftUI and later by Tauri. The engine is the hard ~60% of the work and must not be rewritten per platform.

---

## Decisions

### Core Engine in Rust
**Decision:** Write the core engine as a UI-agnostic Rust library crate.

**Rationale:**
- Rust is reusable by SwiftUI (via UniFFI) **and** native to Tauri.
- Gives memory safety and performance suited to the project budgets.

### Swift Binding via UniFFI
**Decision:** Expose the Rust engine to the SwiftUI shell via **UniFFI** bindings.

**Rationale:**
- Automates boilerplate bridge code, exposing a safe and clean Swift API.

---

## Consequences

- The cross-platform/Tauri pivot only requires rebuilding the shell + UI; the engine logic carries over untouched.
- Adds an FFI boundary on macOS (managed via UniFFI) and a learning investment if Rust is new — offset by strong reuse.
- **Alternatives Considered:**
  - *C++ engine* (works with Swift, but awkward in the Tauri leg).
  - *TypeScript-only* (cannot meet performance budgets).
