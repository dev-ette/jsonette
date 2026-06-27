# 4. macOS editor selection with fallback

Date: 2026-06-27
## Status
Accepted

---

## Context

The editor is the crown jewel. Native macOS has no drop-in equal to the web's CodeMirror. Options ranged from low-level text primitives to turnkey editors. jsonette only needs one language (JSON) and already owns the "smart" parts in the engine.

---

## Decisions

### Primary: CodeEditSourceEditor
**Decision:** Use **`CodeEditSourceEditor`** (from CodeEditApp, MIT, tree-sitter) — the editor that powers the open-source CodeEdit IDE — feeding it the engine's diagnostics and completion data.

**Rationale:**
- Out-of-the-box support for highlighting, inline errors, completion, and find/replace.
- Built-in support for tree-sitter.

### Fallback: CodeEditTextView
**Decision:** If `CodeEditSourceEditor` is outgrown on control or performance, drop down to its own primitive **`CodeEditTextView`** (same family, same APIs underneath) rather than switching ecosystems.

**Rationale:**
- Keeps upgrade/downgrade paths straightforward within the same framework family.

---

## Consequences

- Validate early in M1 due to the pre-1.0 status of CodeEdit components.
- The engine/shell separation ([ADR-0003](0003-strict-engine-shell-separation.md)) ensures that swapping the editor view does not impact business logic.
- **Alternatives Considered:**
  - *CodeEditorView (Chakravarty)*: Turnkey but single-maintainer and offers no primitive to drop down to.
  - *Raw STTextView (Krzyżanowski)*: Fine primitive, but not in the same family as a turnkey option.
