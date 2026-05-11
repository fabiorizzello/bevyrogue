# M015 Discussion Log

## Exchange — 2026-05-08T09:43:58.196Z

### Architecture

When M015 finds mixed combat patterns, how aggressively should it normalize them?

- **Normalize clear drift (Recommended)** — Fix duplicated or disconnected logic when it violates the kernel/RON/hook/presentation model, but stop before a broad rewrite.
- **Only blockers** — Fix only what blocks tests, CLI proof, or validation closure; document deeper architecture drift for later.
- **Full normalization** — Treat all mixed patterns as in-scope even if this touches many combat modules and expands M015.

**Selected:** Normalize clear drift (Recommended)

### Blueprint

What should count as the canonical 'Digimon blueprint-like' extension surface for M015?

- **Typed Rust hooks (Recommended)** — Use Rust hook/system modules per mechanic or Digimon line, driven by RON semantic tags and kernel transitions.
- **RON script-like DSL** — Push more unique behavior into RON effects/requests, with Rust interpreting richer declarative scripts.
- **Hybrid per case** — Choose RON or Rust per mechanic, accepting some variation if tests and docs make authority clear.

**Selected:** Typed Rust hooks (Recommended)
**Notes:** cosa intendi per combat kerneltransition? secondo me ron -> custom signals to unique digimon logic module -> hooks into combat kernel

---
## Exchange — 2026-05-08T09:50:14.357Z

### Errors

For M015 failure modes, should we go deep or apply sensible defaults?

- **Use defaults (Recommended)** — I will define practical defaults for stale tests, audit findings, CLI failures, and architecture blockers, then gate the summary.
- **Go deep** — We will discuss failure behavior in detail before planning slices.

**Selected:** Use defaults (Recommended)

---
