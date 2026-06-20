# Contributing to jsonette

Thanks for your interest. jsonette is a small, deliberately-scoped project that values **correctness, stability, and a clean native experience over feature breadth.** Please read this and `docs/ADR.md` before opening a PR.

## The two rules that matter most

1. **Respect the engine/shell boundary (ADR-0003).**
   All logic — parsing, the tree model, formatting, JSONPath, autocomplete data, and error diagnostics — lives in the **Rust engine**. Shells (SwiftUI now, possibly Tauri later) **only render**. Do not add parsing or validation in Swift "because it's quick." If you find yourself wanting to, the feature belongs in the engine.

2. **Stay in scope.** jsonette is an *editor / viewer / query* tool for JSON. It is intentionally **not** a 30-tool bundle. Proposals to add unrelated utilities (HTTP client, regex tester, JWT, etc.) will be declined. Converters and a test-data generator are the only planned expansions, and they live in the engine.

## Project shape

```
engine/    Rust core crate (UI-agnostic) + UniFFI bindings  ← most logic PRs go here
macos/     SwiftUI app                                       ← rendering / UX PRs
docs/      ADRs and architecture notes
.github/   CI, issue/PR templates
```

## Before you start

- Check the **Roadmap** (`ROADMAP.md`) and the milestone labels. Early on, most work is the **M0 backlog**.
- Look for `good first issue`.
- For anything architectural, open an issue first — significant decisions are recorded as **ADRs**, so we discuss before we build.

## Building (once code exists)

- **Engine:** standard `cargo build` / `cargo test`. Run `cargo clippy` and `cargo fmt` before pushing.
- **macOS app:** open the `macos/` Xcode project; it links the engine via UniFFI (build step documented in M0).
- The CI **performance gate** must stay green — see the budgets in `ROADMAP.md`. PRs that regress cold start, idle RAM, large-file handling, or typing latency will be flagged.

## Pull requests

- One focused change per PR. Reference the issue and milestone.
- Add/adjust tests: engine logic needs `cargo test` coverage (property tests with `proptest` where round-trips apply).
- Update docs/ADRs if your change alters a decision.
- Keep commits clean and messages descriptive.

## Quality bar

- Engine changes: prefer correctness and clear types; the public API is consumed across an FFI boundary, so keep it stable and well-shaped.
- UI changes: match native macOS conventions — keyboard-first, proper menus/shortcuts, accessibility.
- Privacy is a feature: **no network calls, no telemetry.** PRs introducing either will be rejected.

## Code of conduct

Be respectful and constructive. See `CODE_OF_CONDUCT.md`.
