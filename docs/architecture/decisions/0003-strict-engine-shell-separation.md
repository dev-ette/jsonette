# 3. Strict engine/shell separation

Date: 2026-06-27
## Status
Accepted

---

## Context

Reuse across shells (and the cheap Tauri gate) depends on logic not leaking into platform UI code.

---

## Decisions

### Core Logic in Engine
**Decision:** The engine owns parsing, document/tree model, formatting, JSONPath query evaluation, autocomplete schema inference, and error positions/diagnostics.

**Rationale:**
- Keeps the core logic 100% portable and shell-agnostic.

### Rendering and UX in Shell
**Decision:** The shell owns the view layer only, including the editor's syntax coloring (native tree-sitter on macOS, CodeMirror in Tauri).

**Rationale:**
- UI components (coloring, scroll offsets, cursor state) are highly platform-specific. Diagnostics are *rendered* by the shell but *computed* by the engine.

---

## Consequences

- Everything on the engine side survives a shell swap verbatim.
- The editor component becomes "just a view," making it replaceable (de-risking [ADR-0004](0004-macos-editor-selection-with-fallback.md)).
- Requires resisting the temptation to do "quick" parsing/validation in Swift.
