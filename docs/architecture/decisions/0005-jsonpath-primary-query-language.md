# 5. JSONPath as the primary query language

Date: 2026-06-27
## Status
Accepted

---

## Context

Users think in path expressions like `data.users[0].name`. The engine can produce autocomplete by walking the live document. A standard, embeddable Rust implementation is desirable.

---

## Decisions

### Primary Query Language: JSONPath
**Decision:** Ship **JSONPath** (RFC 9535, via `serde_json_path`) as the primary query language.

**Rationale:**
- Familiar to users.
- Standardized under RFC 9535.
- Autocomplete maps naturally to keys-at-path from the engine.

---

## Consequences

- Autocomplete works smoothly out of the box.
- JMESPath or jq-style querying can be layered later without disrupting the primary JSONPath experience.
