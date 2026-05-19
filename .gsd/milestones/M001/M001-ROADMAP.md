# M001: Animation asset pipeline foundation

**Vision:** Port and adapt the existing M022 asset-pipeline plan into a generic, roster-ready animation module with typed clip.ron and anim_graph.ron, boot-time validation, adapter-based cross-asset checks, and real windowed hot-reload proof.

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: cargo test loads an Agumon anim_graph.ron as a typed asset through the new animation module and rejects out-of-vocabulary schema values with typed errors.

- [x] **S02: S02** `risk:medium` `depends:[]`
  > After this: cargo test loads Agumon clip.ron as a typed asset and proves geometry parity with the source atlas data.

- [x] **S03: S03** `risk:high` `depends:[]`
  > After this: Valid graph+clip assets pass required checks; broken fixtures fail with typed diagnostics; cross-asset checks use adapter-provided catalogs.

- [x] **S04: S04** `risk:medium` `depends:[]`
  > After this: Non-Agumon animation assets validate through the same generic path, and cargo run --features windowed proves manual hot reload without crash or corrupted world state.

## Boundary Map

Not provided.
