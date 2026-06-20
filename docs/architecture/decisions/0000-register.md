# Architecture Decision Records — jsonette

Numbered, append-only record of significant decisions. Each: Status · Context · Decision · Consequences · Alternatives.
New decisions get a new ADR; superseded ones are marked, not deleted.

---

## ADR-0001 — Native per-OS, macOS first, with a Tauri comparison gate
**Status:** Accepted

**Context.** The app must be highly optimized, stable, and native-feeling. Speed-to-ship and contributor-pool size are explicitly *not* priorities. The team is willing to do more work for a better result.

**Decision.** Build a fully native macOS app (SwiftUI) through v1.0 (M0–M4). Then build a Tauri + Web UI spike that reuses the same engine, measure it, and decide whether to pivot to Tauri for cross-platform or continue with native shells per OS (Linux → Windows).

**Consequences.** Best-possible native macOS result first. The native v1 hardens requirements and architecture, making any later rewrite faster. Requires discipline at the engine/shell boundary (ADR-0003) for the gate to be cheap. The gate must be judged on "is Tauri *close enough* to justify cross-platform leverage," **not** "did Tauri beat native on macOS" (it won't).

**Alternatives.** Tauri-first (rejected: re-introduces a webview + web-UI layering the team wants to avoid, and the editor would not be measurably native). Qt + QScintilla (rejected: GPL-only license, non-native aesthetics — see ADR-0007). Flutter (rejected: its value is identical-UI-everywhere, which is explicitly unwanted).

---

## ADR-0002 — Engine written in Rust, shared across shells via UniFFI
**Status:** Accepted

**Context.** Under ADR-0001 the engine may be consumed first by SwiftUI and later by Tauri. The engine is the hard ~60% of the work and must not be rewritten per platform.

**Decision.** Write the engine as a UI-agnostic Rust crate. Expose it to Swift via **UniFFI**. In a future Tauri shell, the same crate is the native backend — no rewrite.

**Consequences.** The cross-platform/Tauri pivot only requires rebuilding the shell + UI. Rust gives memory safety and performance suited to the budgets. Adds an FFI boundary on macOS (managed via UniFFI) and a learning investment if Rust is new — offset by strong reuse and portfolio value.

**Alternatives.** C++ engine (works with Swift, but awkward in the Tauri leg). TypeScript-only (cannot meet the performance budgets).

---

## ADR-0003 — Strict engine/shell separation
**Status:** Accepted

**Context.** Reuse across shells (and the cheap Tauri gate) depends on logic not leaking into platform UI code.

**Decision.** The **engine owns** parse, document/tree model, formatting, the JSONPath evaluator, autocomplete schema inference, and **error positions/diagnostics**. The **shell owns** the view layer only — including the editor's syntax *coloring* (native tree-sitter on macOS; CodeMirror `lang-json` in Tauri). Diagnostics are *rendered* by the shell but *computed* by the engine in every shell.

**Consequences.** Everything on the engine side survives a shell swap verbatim. The editor component becomes "just a view," making it replaceable (de-risks ADR-0004's pre-1.0 caveat). Requires resisting the temptation to do "quick" parsing/validation in Swift.

---

## ADR-0004 — macOS editor: CodeEditSourceEditor (with CodeEditTextView as fallback)
**Status:** Accepted

**Context.** The editor is the crown jewel. Native macOS has no drop-in equal to the web's CodeMirror. Options ranged from low-level text primitives to turnkey editors. jsonette only needs one language (JSON) and already owns the "smart" parts in the engine.

**Decision.** Use **`CodeEditSourceEditor`** (CodeEditApp, MIT, tree-sitter) — the editor that powers the open-source CodeEdit IDE — feeding it the engine's diagnostics and completion data. If it's outgrown on control or performance, **drop down to its own primitive `CodeEditTextView`** (same family, same APIs underneath) rather than switching ecosystems.

**Consequences.** Get highlighting, inline errors, completion, find/replace, and large-doc handling without building them; needs only the JSON tree-sitter grammar. A graceful degradation path exists within one family. **Caveat:** it is currently pre-1.0 — validate early in M1; the engine/shell split (ADR-0003) keeps it swappable.

**Alternatives.** `CodeEditorView` (Chakravarty) — turnkey but single-maintainer and offers no primitive to drop to. Raw `STTextView` (Krzyżanowski) — a fine primitive, but not in the same family as a turnkey option, so no graceful upgrade/downgrade path.

---

## ADR-0005 — JSONPath as the primary query language
**Status:** Accepted

**Context.** Users think in path expressions like `data.users[0].name`. The engine can produce autocomplete by walking the live document. A standard, embeddable Rust implementation is desirable.

**Decision.** Ship **JSONPath** (RFC 9535, via `serde_json_path`) as the primary query language. JMESPath may be added later as a second dialect; jq-style power querying is a possible future option.

**Consequences.** Familiar to users; autocomplete maps naturally to keys-at-path from the engine. JMESPath/jq can be layered later without disrupting the primary experience.

---

## ADR-0006 — Project name: jsonette
**Status:** Accepted

**Context.** Wanted a memorable, easy-to-type, relevant name with clean namespaces (the Rust engine publishes to crates.io). Candidates `jsonit` and `jsonix` had collisions: `jsonit` is taken on crates.io by an in-domain JSON crate (and on npm + GitHub); `jsonix` has a free crate name but a taken npm name held by an existing XML↔JSON library (which overlaps the planned converter feature), plus a taken GitHub org.

**Decision.** Use **jsonette** — verified free on crates.io, npm, GitHub org, and Homebrew cask. The "-ette" diminutive also signals lightweight, matching the product thesis.

**Consequences.** No namespace workarounds needed. Reserve crates.io + GitHub org promptly.

---

## ADR-0007 — License: MIT OR Apache-2.0 (dual)
**Status:** Accepted

**Context.** Open-source distribution; maximum adoption desired; want a patent grant. The Qt + QScintilla path (considered and rejected) would have forced GPL-v3.

**Decision.** Dual-license under **MIT OR Apache-2.0**, the Rust-ecosystem convention.

**Consequences.** Broad reuse, patent protection via Apache-2.0, no copyleft constraints. Available precisely because Qt/QScintilla was not chosen.
