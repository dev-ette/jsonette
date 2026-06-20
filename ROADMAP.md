# jsonette — Roadmap

Milestones, exit criteria, and the M0 backlog. Pairs with `PLANNING.md`.
Sizing is relative (S / M / L), not calendar time — this is a solo + trusted-contributors project.

---

## Phase 1 — Native macOS to v1.0

| # | Milestone | Exit criteria | Size |
|---|---|---|---|
| **M0** | Foundations | Repo, license, CI, signing pipeline, engine + app scaffolds, UniFFI bridge proven. See backlog below. | M |
| **M1** | Editor | `CodeEditSourceEditor` integrated; tree-sitter JSON coloring; engine-driven error diagnostics; format/minify; 50 MB file stays interactive; first signed `.dmg`. | L |
| **M2** | Tree Viewer | `NSOutlineView` virtualized tree; datatype rendering; search + jump; editor↔tree two-way sync. | M |
| **M3** | Query Tool | JSONPath via engine; native autocomplete from live schema; span-accurate error messages; results pane + history. | L |
| **M4** | **macOS 1.0** | UX polish; all perf budgets green in CI; notarized `.dmg`; **Homebrew cask** published; tagged GitHub release with notes. | M |

## Phase 2 — Decision gate

| # | Milestone | Exit criteria |
|---|---|---|
| **G1** | Tauri spike | Tauri + CodeMirror shell over the *same* Rust engine, at parity with M1–M3 core; measured against the budgets below. |
| **G2** | Go / no-go | Written comparison + decision, judged by the §2 gate criterion in `PLANNING.md` (not "did it beat native on macOS"). |

## Phase 3 — Expansion (branch depends on G2)

- **If Tauri passes:** port to **Linux**, then **Windows**, from one Tauri codebase; publish to Flatpak/apt and winget.
- **If not:** native shells per OS over the same engine — **Linux first** (high developer adoption), **Windows last** (largest market, but the weakest native code-editor story → defer the hardest lift).

## Parked / later
- **Converters** (JSON ↔ YAML / XML / TOML / CSV) — engine feature; architecture already accommodates it.
- **SQL-schema → test-data generator** (`sqlparser-rs` + `fake`/`rand`) — engine feature.
- **Mobile** — only if an experienced native mobile dev joins; shared engine + separate native UI.

---

## Performance budgets (CI-enforced; also the gate thresholds)

- Cold start **< 400 ms**
- Idle RAM **< 60 MB**
- Open + render 50 MB JSON **< 1.5 s**, then 60 fps scroll
- No main-thread block **> 16 ms** while typing
- macOS bundle **< 15 MB**

**Gate margins (lock before G1):** Tauri passes if e.g. idle RAM ≤ 2× and cold start ≤ 1.5× the native numbers, under absolute ceilings.

---

## M0 backlog (seed these as GitHub issues)

### Repo & governance
- [ ] Create GitHub org/repo `jsonette`; reserve crates.io name `jsonette`.
- [ ] Add dual `LICENSE-MIT` + `LICENSE-APACHE`; reference both in README.
- [ ] Add `README.md`, `PLANNING.md`, `ROADMAP.md`, `CONTRIBUTING.md`, `docs/ADR.md`, `CODE_OF_CONDUCT.md`.
- [ ] Issue + PR templates; labels (`M0`…`M4`, `good first issue`, `engine`, `macos`, `perf`).
- [ ] GitHub Projects board; map milestones M0–M4.

### Engine scaffold (Rust)
- [ ] `engine/` crate; CI `cargo build`/`test`/`clippy`/`fmt`.
- [ ] Define the public API surface the shell will consume (parse → model, format, query, diagnostics, completion). Keep it UI-agnostic.
- [ ] Wire `serde_json`; spike a tolerant parse path for the editor.
- [ ] Add `proptest` round-trip tests (parse↔serialize, format idempotence).

### Bridge
- [ ] Prove **UniFFI** end-to-end: call a trivial engine function from Swift and return a typed result.
- [ ] Document the bridge build step so contributors can reproduce it.

### macOS app scaffold (SwiftUI)
- [ ] `macos/` Xcode project; SwiftUI app shell that links the engine via UniFFI.
- [ ] Open-a-file flow → engine parse → render raw text (no editor yet). Smoke test of the full path.

### Signing & release pipeline (do this early — never let it block M4)
- [ ] Apple Developer ID set up; Developer ID Application certificate in CI secrets.
- [ ] GitHub Actions: build → sign → **notarize** → staple → produce `.dmg` on tag.
- [ ] Verify notarization on an *empty* app before there's anything to lose.

### Performance harness
- [ ] CI job that fails the build if a budget (above) is exceeded.
- [ ] Generate fixture JSON files (1 MB / 10 MB / 50 MB) for benchmarks.

**M0 is plumbing, not features.** It is "done" when an empty, signed, notarized app opens a JSON file through the Rust engine, and CI enforces the budgets.
