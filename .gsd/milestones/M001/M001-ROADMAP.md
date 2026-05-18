# M001: Animation asset pipeline foundation

**Vision:** Port and adapt the existing M022 asset-pipeline plan into a generic, roster-ready animation module with typed `clip.ron` and `anim_graph.ron`, boot-time validation, adapter-based cross-asset checks, and real `windowed` hot-reload proof. M022 is the scope seed, not a rigid implementation law.

## Success Criteria

- `cargo test` proves typed `anim_graph.ron` and `clip.ron` loading plus validator behavior for valid and broken fixtures.
- Agumon proves the full real-data path, including geometry parity for `clip.ron`.
- Non-Agumon support validates through the same generic architecture without Digimon-specific engine hardcoding.
- Cross-asset checks use explicit adapters rather than direct animation-core coupling to gameplay or Digimon data internals.
- Manual `cargo run --features windowed` hot-reload proof is completed and documented.

## Slices

- [ ] **S01: Animation module and anim graph schema** `risk:high` `depends:[]`
  > After this: `cargo test` loads an Agumon `anim_graph.ron` as a typed asset through the new animation module and rejects out-of-vocabulary schema values with typed errors.

- [ ] **S02: Clip schema and lossless geometry loading** `risk:medium` `depends:[]`
  > After this: `cargo test` loads Agumon `clip.ron` as a typed asset and proves geometry parity with the source atlas data.

- [ ] **S03: Validator L with adapter based checks** `risk:high` `depends:[S01,S02]`
  > After this: Valid graph+clip assets pass required checks; broken fixtures fail with typed diagnostics; cross-asset checks use adapter-provided catalogs.

- [ ] **S04: Roster ready assets and real hot reload proof** `risk:medium` `depends:[S01,S02,S03]`
  > After this: Non-Agumon animation assets validate through the same generic path, and `cargo run --features windowed` proves manual hot reload without crash or corrupted world state.

## Boundary Map

### S01 → S03

Produces:
- Generic animation module seam and typed `AnimGraph` schema.
- Closed graph vocabulary for nodes, edges, predicates, commands, parameter references, and target shapes.
- Loader registration/lifecycle for `anim_graph.ron`.

Consumes:
- nothing

### S02 → S03

Produces:
- Typed `Clip` schema and loader.
- Agumon `clip.ron` geometry parity proof against source atlas data.

Consumes:
- nothing

### S03 → S04

Produces:
- Validator API and typed diagnostic behavior.
- Adapter catalog seam for project-specific cross-asset checks.

Consumes:
- `AnimGraph` from S01.
- `Clip` from S02.

### S04 → milestone exit

Produces:
- Roster-ready asset coverage through the same generic path.
- Documented manual hot-reload UAT evidence.

Consumes:
- typed assets and validator from S01, S02, and S03.
