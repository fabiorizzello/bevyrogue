---
phase: M001
phase_name: Animation asset pipeline foundation
project: bevyrogue
generated: 2026-05-19T00:00:00Z
counts:
  decisions: 5
  lessons: 3
  patterns: 4
  surprises: 3
missing_artifacts:
  - S03 task-level SUMMARY.md files (all four blank, verification_result untested)
  - Live cargo run --features windowed session output (procedure documented but not captured)
---

### Decisions

- **Closed serde enums for animation schema vocabulary.** Unknown RON field values (node types, commands, predicates, transition targets) fail at parse time via closed enum deserialisation, not at runtime. `TransitionTarget::Node(…) | Exit` avoids ambiguous untagged decoding.
  Source: S01-SUMMARY.md/Key decisions

- **All animation types under a single `src/animation` module seam.** AnimGraph and Clip are co-located under `src/animation` rather than scattered through Digimon-specific or data modules. This enforces a clear generic boundary.
  Source: S01-SUMMARY.md/Patterns established

- **Asset readiness gated on `Assets<T>` read success, not just load events.** The pattern for Bevy typed asset loaders in this project: readiness flips to `true` only after a confirmed `Assets<T>` lookup, not merely an `AssetEvent::Added`. This prevents false-ready states.
  Source: S01-SUMMARY.md/Key decisions; S02-SUMMARY.md/Patterns established

- **Cross-asset validation through explicit adapter injection, not direct coupling.** The animation validator receives project data (skill catalogs, status names, particle names) via adapter structs rather than importing Digimon data types directly. Tests inject real Agumon data through the same adapters.
  Source: S03-SUMMARY.md/What Happened

- **Dynamic roster discovery in AnimationAssetPlugin — no per-character registration.** The plugin scans asset paths at startup to discover roster entries rather than hardcoding Agumon paths. Adding a new Digimon requires only placing the asset files; no plugin code changes.
  Source: S04-SUMMARY.md/patterns_established

### Lessons

- **Authoritative parity tests against source JSON are mandatory for authored asset geometry.** `clip.ron` was authored in S02 with wrong `frame_size` (w=557 vs w=512) and systematic off-by-one ranges from `heavy_attack` onward. Because the values were structurally valid RON, no parse error occurred. The dedicated `clip_geometry_parity` test comparing `clip.ron` directly against `agumon_atlas.json` was the only thing that caught the drift. Without it, wrong geometry would have shipped as the milestone foundation. Write atlas-parity tests before marking clip authoring complete.
  Source: S04-SUMMARY.md/Known Limitations; M001-VALIDATION.md/Slice Delivery Audit

- **Task-level SUMMARY.md files must be written before slice closeout.** All four S03 task SUMMARY files were left blank (`verification_result: untested`, no evidence recorded). The slice closed as `passed` at the slice level, but the task-level documentation traceability is broken. Future milestones depending on S03 decisions cannot trace back to specific tasks. Enforce task summary completion as a slice closeout gate.
  Source: M001-VALIDATION.md/Slice Delivery Audit

- **Operational UAT requires captured live output, not just procedure documentation.** S04-UAT documents the full `cargo run --features windowed` hot-reload procedure with preconditions, steps, and edge cases, but no actual live session output was captured. A documented procedure without evidence does not satisfy an Operational verification class. Future hot-reload UAT must include pasted console logs or screen-captured evidence alongside the procedure.
  Source: M001-VALIDATION.md/Requirement Coverage; M001-VALIDATION.md/Verification Class Compliance

### Patterns

- **Generic typed asset loaders in Bevy gate readiness on both asset events and `Assets<T>` reads.** First receive the `AssetEvent::Added`, then confirm the asset is retrievable from `Assets<T>`; only then set the readiness flag. This guards against transient states where the event fires before the asset is fully registered.
  Source: S01-SUMMARY.md/Patterns established; S02-SUMMARY.md/Patterns established

- **Authoritative source-data parity tests assert exact geometry plus inclusive range semantics.** For any animation asset authored from a sprite-atlas JSON, write a dedicated integration test that loads both files and asserts frame_size, columns, rows, total_frames, and every named range start/end against the atlas. Use inclusive-end semantics consistent with how RON ranges are interpreted.
  Source: S02-SUMMARY.md/Patterns established

- **Adapter-injected cross-asset validation keeps the animation core decoupled from Digimon internals.** The validator struct accepts skill catalogs, status names, and particle names through adapter types. This keeps `src/animation` free of any `import bevyrogue::data` dependency and allows test isolation via injected stubs or real production adapters.
  Source: S03-SUMMARY.md/What Happened

- **Dynamic roster discovery via asset-path scanning removes per-character plugin registration.** The `AnimationAssetPlugin` discovers all roster entries by scanning a known asset directory tree at startup. Adding a new Digimon is a data-only change (place `clip.ron` + `anim_graph.ron`), not a code change. This pattern should be applied to any future roster-scoped plugin.
  Source: S04-SUMMARY.md/patterns_established

### Surprises

- **Wrong clip.ron geometry produced no parse error — only the parity test caught it.** The S02-authored `clip.ron` had `frame_size: (w: 557, h: 561)` and `total_frames: 95`, both perfectly valid RON numbers. No deserialisation failure, no runtime panic. The bug was invisible until S04 surfaced it via the parity test. This is a category of silent authoring error that static typing alone cannot prevent.
  Source: S04-SUMMARY.md/Known Limitations; MEM008

- **Correcting clip.ron ranges cascaded into anim_graph.ron frame references.** When clip.ron ranges were shifted down by 1–2 frames, the `baby_flame_cast`, `baby_flame_impact`, and `baby_flame_recover` nodes in `anim_graph.ron` also needed updating because their `frames` fields index into the clip's frame namespace. This coupling was not obvious from the schema definitions alone.
  Source: git diff HEAD -- assets/digimon/agumon/anim_graph.ron

- **S03 task summaries were silently left blank despite slice passing.** The GSD slice-completion tool does not currently enforce that task-level SUMMARY.md files are non-empty before allowing slice closeout. All four S03 task summaries reached `verification_result: untested` with no prose. This gap in the tooling allows documentation debt to accumulate invisibly.
  Source: M001-VALIDATION.md/Slice Delivery Audit
