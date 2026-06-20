# jsonette

> A focused, fast, native JSON editor, viewer, and query tool. Lightweight by design. Local-only. Open source.

**Status:** 🟡 Planning (pre-M0). No code yet — this repo currently holds the architecture and project plan.
**License:** MIT OR Apache-2.0
**First platform:** macOS (native, SwiftUI). Linux and Windows to follow (see Roadmap).

---

## What jsonette is

A small desktop app that does three things better than anyone, instead of thirty things adequately:

1. **Edit** — a real code-editor experience for JSON: syntax highlighting, error highlighting, auto-format, stable on large files.
2. **View** — a collapsible tree with click-to-navigate, collapse, search, and datatype-aware rendering (object, array, string, number, boolean, null).
3. **Query** — JSONPath with intelligent autocomplete, inline syntax/error highlighting, and a live results pane.

## What jsonette is **not**

No HTTP client, no regex tester, no JWT decoder, no color pickers, no port scanner. Feature count is not the goal — **focus, native quality, privacy, and being free are.** Bundled "developer Swiss-army-knife" apps already exist; jsonette is the opposite bet.

## Why it exists

- **Free & open source** — auditable, no license fees.
- **Private by design** — 100% local processing, zero network calls, zero telemetry. Your JSON never leaves your machine. (A real feature for anyone pasting sensitive data.)
- **Lightweight** — strict, CI-enforced performance budgets (see `ROADMAP.md`).
- **Cross-platform path** — one portable engine, native shells; macOS first, then Linux, then Windows.

## Architecture in one picture

```
        ┌──────────────────────────────────────────────┐
        │  ENGINE (Rust crate, zero UI deps)            │
        │  parse · tree model · format · JSONPath ·     │
        │  autocomplete data · diagnostics              │
        └───────────────────────┬──────────────────────┘
                                │  written once, reused everywhere
          ┌──────────────────────┴───────────────────────┐
          ▼                                               ▼
   macOS shell (SwiftUI)                          (later) Tauri shell
   via UniFFI                                     Rust engine is native here
```

The **engine owns all the logic**; each **shell only renders**. This is what makes the platform expansion (and the later Tauri performance comparison) cheap — the hard 60% carries over untouched. See `PLANNING.md` §3.

## Tech stack (v1, macOS)

| Layer | Choice |
|---|---|
| Engine | Rust (`serde_json`, `serde_json_path`), exposed to Swift via UniFFI |
| UI | SwiftUI (+ AppKit where needed), macOS 14+ |
| Editor | `CodeEditSourceEditor` (tree-sitter), with `CodeEditTextView` as the lower-level fallback |
| Tree | `NSOutlineView` (lazy, virtualized) |
| Packaging | signed + notarized `.dmg`, Homebrew cask |

Full rationale and alternatives in `docs/architecture/decisions/0000-register.md`.

## Planned repo structure

```
jsonette/
├── engine/        # Rust core crate (UI-agnostic) + UniFFI bindings
├── macos/         # SwiftUI app (Xcode project)
├── docs/          # ADRs, architecture notes
├── .github/       # CI workflows, issue/PR templates
├── README.md
├── PLANNING.md    # architecture + strategy (master plan)
├── ROADMAP.md     # milestones + backlog
└── CONTRIBUTING.md
```

## Roadmap at a glance

- **Phase 1 — macOS to v1.0:** M0 foundations → M1 editor → M2 viewer → M3 query → M4 release (Homebrew).
- **Decision gate:** build a Tauri spike reusing the same engine; measure; go/no-go on a cross-platform pivot.
- **Phase 3 — expansion:** Linux → Windows (either via Tauri, or native shells over the same engine).
- **Later:** converters (JSON ↔ YAML/XML/TOML/CSV), SQL-schema → test-data generator.
- **Parked:** mobile (only if an experienced native mobile dev joins).

See `ROADMAP.md` for milestone exit criteria and the M0 backlog.

## Contributing

This is a small, deliberately-scoped project that values correctness and stability over feature breadth. Read `CONTRIBUTING.md` and the ADRs before opening a PR — especially the **engine/shell separation** rule, which is the backbone of the whole design.

## License

Dual-licensed under MIT or Apache-2.0, `LICENSE-MIT`, and `LICENSE-APACHE`
