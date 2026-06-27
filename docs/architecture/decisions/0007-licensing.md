# 7. Licensing

Date: 2026-06-27
## Status
Accepted

---

## Context

We need an open-source distribution model that encourages maximum adoption, offers a patent grant, and remains compatible with the Rust ecosystem.

---

## Decisions

### Selected Licenses: MIT OR Apache-2.0
**Decision:** Dual-license the project under **MIT OR Apache-2.0**.

**Rationale:**
- Conforms to standard Rust ecosystem conventions.
- Offers broad reuse and patent protection (via Apache-2.0) with no copyleft constraints.

---

## Consequences

- Compatible with commercial use.
- Avoids GPL-v3 copyleft constraints (which would have been forced if Qt/QScintilla was selected).
