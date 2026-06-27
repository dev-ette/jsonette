# 1. Native per-OS, macOS first, with a Tauri comparison gate

Date: 2026-06-27
## Status
Accepted

---

## Context

The app must be highly optimized, stable, and native-feeling. Speed-to-ship and contributor-pool size are explicitly *not* priorities. The team is willing to do more work for a better result.

---

## Decisions

### Native macOS SwiftUI Shell
**Decision:** Build a fully native macOS app (SwiftUI) through v1.0 (M0–M4).

**Rationale:**
- Best possible native macOS result.
- Enforces strict native controls and performance over bloated multi-gigabyte Electron apps.

### Tauri Comparison Gate
**Decision:** Build a Tauri + Web UI spike that reuses the same Rust engine, measure it, and decide whether to pivot to Tauri for cross-platform or continue with native shells per OS (Linux → Windows).

**Rationale:**
- The gate must be judged on "is Tauri *close enough* to justify cross-platform leverage," **not** "did Tauri beat native on macOS" (it won't).

---

## Consequences

- Requires discipline at the engine/shell boundary ([ADR-0003](0003-strict-engine-shell-separation.md)) for the gate to be cheap.
- **Alternatives Considered:**
  - *Tauri-first* (rejected: re-introduces webview + web-UI layering, and editor wouldn't feel native).
  - *Qt + QScintilla* (rejected: GPL-only license, non-native aesthetics — see [ADR-0007](0007-licensing.md)).
  - *Flutter* (rejected: its value is identical-UI-everywhere, which is unwanted).
