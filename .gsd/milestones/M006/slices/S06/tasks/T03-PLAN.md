---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Warn-once on unbuildable graph event

Emit a single warn log (deduplicated per graph handle) when a graph asset event cannot produce a registry entry, giving the handle/path for diagnosis. No secrets, no per-frame spam.

## Inputs

- `src/animation/registry.rs`

## Expected Output

- `Deduplicated warn on spawn-miss; silent on the happy path`

## Verification

cargo test (headless green); manual: cargo winx shows no warn for Renamon/Agumon happy path
