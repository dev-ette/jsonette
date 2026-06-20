# jsonette — Project Plan & Architecture

> A focused, lightweight JSON editor / viewer / query tool. Open source, local-only, enterprise-grade.
> **Strategy: native macOS first (to v1.0) → a measured Tauri comparison gate → expand.**

This is the master plan. Companion docs: `ROADMAP.md` (milestones + backlog), `docs/ADR.md` (decision records), `README.md` (public overview).

---

## 1. Vision & Scope

jsonette does three things better than anyone, instead of thirty adequately:

1. **Edit** JSON — syntax highlighting, error highlighting, auto-format, stable large-file editing.
2. **View** JSON — collapsible tree, navigation, search, datatype-aware rendering.
3. **Query** JSON — JSONPath (primary) with autocomplete, syntax/error highlighting, live results.

**Out of scope (deliberately):** HTTP client, regex tester, JWT, color tools, and the rest of the "30-tool bundle" model. The differentiators are **focus + open source + privacy + native quality** — not feature count.

**Edge over bundled competitors:** free, open source, auditable, zero telemetry, and a cross-platform path — versus paid, closed, single-platform, kitchen-sink tools.

---

## 2. Strategy: Two Phases with a Decision Gate

```
PHASE 1  ───────────────────────────────────────────►  v1.0 (macOS, native)
   Rust engine + native SwiftUI macOS shell.
   Ship a polished, signed, notarized macOS 1.0.  (M0–M4)
                         │
                         ▼
            ┌────────────────────────────┐
            │  DECISION GATE              │
            │  Build a Tauri + Web UI     │
            │  spike reusing the SAME     │
            │  Rust engine. Measure.      │
            └─────────────┬──────────────┘
                          │
        ┌─────────────────┴──────────────────┐
        ▼                                     ▼
  Tauri "good enough"                   Tauri not worthy
  → pivot to Tauri for                  → native shells per OS over
    cross-platform                        the same engine,
    (one codebase:                        market order: Linux → Windows
     mac/linux/windows)
```

### Decision-gate success criterion (the part that's easy to get wrong)
On **macOS alone**, native will beat Tauri on raw performance — no webview, no IPC. That is expected and is **not** the test. Tauri's value is platforms 2 and 3, where one codebase replaces two more native rewrites.

> **The gate question is NOT "did Tauri beat native on macOS?"** (it won't)
> **It is "is Tauri's performance *close enough* that one-codebase cross-platform is worth the small native-feel/perf cost?"**

Commit the pass/fail thresholds (see §5) **before** running the spike, so the decision is objective.

### Why this strategy is efficient
The Phase-2 rewrite is cheap because **the engine carries over untouched** — only the shell + UI is rebuilt. The native v1 also hardens the requirements and architecture, so the second build is faster and better-shaped than the first. This efficiency depends entirely on the engine being in Rust (§4) and on a disciplined engine/shell boundary (§3).

---

## 3. Architecture: Engine / Shell Separation

```
┌──────────────────────────────────────────────────────────┐
│  ENGINE — Rust crate, zero UI dependencies                 │
│  Owns everything that must survive the native→Tauri pivot: │
│   • JSON parse + tolerant/incremental parse                │
│   • Document & tree model                                  │
│   • Formatter / pretty-print / minify                      │
│   • JSONPath evaluator (+ JMESPath later)                  │
│   • Autocomplete schema inference (keys-at-path)           │
│   • Error positions & diagnostics (byte-offset → line/col) │
│   • Converters (later: YAML/XML/TOML/CSV)                  │
│   • SQL-schema → test-data generator (later)              │
└───────────────────────────┬──────────────────────────────┘
                            │  written ONCE, in Rust
          ┌──────────────────┴───────────────────┐
          ▼                                       ▼
  PHASE 1 SHELL                            PHASE 2 SHELL (gate)
  SwiftUI macOS via UniFFI                 Tauri + Web UI
  Renders: editor view + native            Rust engine is native here
  tree-sitter coloring, NSOutlineView      Renders: CodeMirror + virtualized DOM
```

**The boundary (memorize this — it's the backbone):**
- **Engine owns:** parse, model, format, query, autocomplete data, **diagnostics**. Carries over verbatim across shells.
- **Shell owns:** view-layer only — including the editor's syntax *coloring* (native tree-sitter on macOS; CodeMirror `lang-json` in Tauri). Error squiggles are *rendered* by the shell but *computed* by the engine in both worlds.

If logic leaks into the shell, you pay for it again at every new platform and at the Tauri gate. Hold the line from M0.

---

## 4. Technology Selection

### Engine (both phases)
| Concern | Choice | Why |
|---|---|---|
| **Language** | **Rust** | The one language reusable by SwiftUI (via UniFFI) **and** native to Tauri. Makes the Phase-2 rewrite cheap. Memory-safe, fast, strong portfolio signal. |
| JSON parse | `serde_json` + a tolerant parser for the editor (`tree-sitter-json` or hand-rolled) | Correctness + error-tolerant editing |
| Query | `serde_json_path` (RFC 9535 JSONPath); `jmespath` later | JSONPath-first |
| Swift bridge | **UniFFI** (alt: `swift-bridge`) | Generates a clean Swift API over the Rust engine |
| License | **MIT OR Apache-2.0** | Maximum adoption + patent grant; available now that Qt/GPL is off the table |

### Phase 1 shell — native macOS
| Concern | Choice | Notes |
|---|---|---|
| UI framework | **SwiftUI** (+ AppKit where needed) | macOS 14+ baseline |
| Editor | **`CodeEditSourceEditor`** (CodeEditApp, MIT, tree-sitter) | Full editor used by the CodeEdit IDE: highlighting, inline error messages, completion, find/replace, large-doc handling. Feed it the engine's diagnostics + completion data. **Escape hatch:** drop to its own primitive `CodeEditTextView` (same family) for full control if outgrown. |
| Syntax coloring | `tree-sitter-json` (via the editor) | Fast, incremental, error nodes |
| Tree viewer | **`NSOutlineView`** (wrapped for SwiftUI) | Built for lazy, virtualized hierarchical data — what Finder uses. Best for large JSON. (`OutlineGroup` is fine for small data.) |
| Query autocomplete | Native completion popover driven by the engine's keys-at-path | Feels like "interpreting a JS line" |
| Packaging | signed + notarized `.dmg`, **Homebrew cask** | Mac App Store optional (sandboxing/file-access tradeoffs) |

**Editor caveat (eyes open):** every turnkey native SwiftUI code editor is currently pre-1.0. Mitigated by the engine/shell split — the editor only *renders*, so it's swappable. Validate the pick early in M1.

### Phase 2 shell — Tauri (only if the gate passes)
Tauri 2 + CodeMirror 6 + a lightweight frontend (Svelte/Solid). Reuses the Rust engine natively. Detailed only when the gate is reached.

---

## 5. Non-Functional Requirements & Budgets

### Performance budgets (enforced in CI; also define the Phase-2 gate thresholds)
- Cold start: **< 400 ms**
- Idle RAM: **< 60 MB**
- Open + render 50 MB JSON: **< 1.5 s**, then scroll at 60 fps
- No main-thread block **> 16 ms** while typing
- macOS app bundle: **< 15 MB**

**Gate rule of thumb:** Tauri "passes" if it lands within a pre-agreed margin of these native numbers (e.g. ≤ 2× idle RAM, ≤ 1.5× cold start) while staying under absolute ceilings — i.e. *good enough to justify cross-platform leverage*. Lock exact margins before the spike.

### Enterprise-grade & trust (a real differentiator)
- **100% local. Zero network calls. Zero telemetry.**
- Signed + notarized builds; reproducible builds + SBOM.
- Keyboard-first UX, full macOS shortcut + menu integration, accessibility.

---

## 6. Feature Detail

### Editor
Syntax highlighting + coloring; engine-driven error diagnostics (squiggles + gutter); auto-format/minify; bracket matching, folding, line numbers; large-file strategy (lazy parse, off-main-thread engine work). Optional auto-repair (trailing commas, single quotes) as an engine feature.

### Tree Viewer
Virtualized collapsible tree (`NSOutlineView`); datatype-aware rendering; click-to-navigate, expand/collapse, breadcrumb path; key/value search with highlight + jump; two-way sync with the editor; copy-path-as-JSONPath and copy-value.

### Query Tool
JSONPath input with syntax highlighting; **autocomplete** driven by the engine walking the live document (keys-at-path); span-accurate error messages; live results pane (tree or raw) with result count; query history.

---

## 7. Roadmap (summary)

Full milestone exit criteria and the M0 backlog live in `ROADMAP.md`.

- **Phase 1 (macOS → v1.0):** M0 foundations · M1 editor · M2 viewer · M3 query · M4 release + Homebrew.
- **Gate:** G1 Tauri spike · G2 documented go/no-go.
- **Phase 3:** Linux → Windows (Tauri pivot *or* native shells over the same engine).
- **Post-v1 engine features:** converters; SQL-schema data generator.
- **Parked:** mobile (only with an experienced native mobile dev; shared engine + separate native UI; never Flutter).

---

## 8. Project Management & Governance

- **Monorepo:** `engine/` (Rust) · `macos/` (SwiftUI) · `docs/` (ADRs) · `.github/` (CI, templates). Add `tauri/` only at the gate.
- **Decision trail:** ADRs in `docs/ADR.md` so contributors understand the *why*.
- **Tracking:** GitHub Projects board mapped to milestones; one milestone = one release; M0 backlog seeded as issues.
- **CI/CD:** GitHub Actions — build + test (engine: `cargo test` + `proptest`; macOS: XCTest), perf-budget gate, signed release artifacts on tags.
- **Costs to budget:** Apple Developer Program (~$99/yr) for notarization (+ App Store if pursued).

---

## 9. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Native macOS editor component pre-1.0 / less mature than CodeMirror | Med | Med | Validate in M1; `CodeEditTextView` fallback; engine owns diagnostics so the editor is "just rendering" |
| Engine/shell line drawn wrong → poor carryover at the gate | Med | High | Keep diagnostics/format/query/model strictly in Rust from M0 |
| Gate judged on the wrong criterion (macOS-only perf) | Med | High | Pre-commit §2 gate question + §5 margins before the spike |
| Large-file perf misses budget | Med | High | `NSOutlineView` virtualization + lazy parse from M1; CI benchmarks |
| Scope creep toward "30 tools" | High | High | Focus is the thesis. Say no by default. |

---

## 10. Locked Decisions

- **Name:** **jsonette** (verified free on crates.io, npm, GitHub org, Homebrew cask).
- **Architecture:** pure native per-OS, **macOS first to v1.0**, then Tauri comparison gate.
- **Engine:** **Rust**, reused across shells via UniFFI.
- **Engine/shell boundary:** engine owns all logic incl. diagnostics; shells render only.
- **Query:** **JSONPath-first**.
- **macOS editor:** **`CodeEditSourceEditor`**, `CodeEditTextView` as fallback.
- **Tree:** `NSOutlineView`.
- **License:** **MIT OR Apache-2.0**.
- **Mobile:** **parked**.

*Planning output only. Implementation begins at M0.*
