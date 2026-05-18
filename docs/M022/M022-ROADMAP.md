# M022: Asset pipeline: clip + anim_graph loader, validator, hot-reload (Agumon-only)

**Vision:** I due asset di animazione canon (§2.2 clip.ron geometria lossless, §2.2b anim_graph.ron grafo FSM) caricano come asset Bevy tipizzati via il pattern RonAssetPlugin esistente, validati staticamente al boot dal validator §L, hot-reloadabili. Tutto su Agumon, headless-first. Nessun runtime player (M023). Lo schema provato qui è il contratto su cui M023 costruisce il render.

## Success Criteria

- cargo test carica Agumon clip.ron e anim_graph.ron (baby_flame, forma §M) come asset tipizzati e li valida con l'intero §L verde.
- Un fixture anim_graph.ron rotto per ognuno dei check §L fa fallire il boot con DataError che nomina file + check violato — mai un fallimento a runtime.
- L'Agumon clip.ron è geometricamente identico ad agumon_atlas.json (test di equivalenza lossless).
- cargo run --features windowed con Agumon: editare clip.ron/anim_graph.ron a caldo ricarica l'asset senza panic né world-state corrotto.
- cargo check headless e --features windowed puliti; nessun import winit/wgpu/egui fuori dal feature gate.

## Slices

- [ ] **S01: AnimGraph typed schema + loader** `risk:high (schema complexity)` `depends:[]`
  > After this: cargo test carica Agumon anim_graph.ron baby_flame (forma §M) come AnimGraph tipizzato; asserzioni su nodi, edge, Predicate §D, Command §C/§C2, ParamRef §S; un RON con vocabolo fuori-enum è rifiutato come DataError tipizzato.

- [ ] **S02: Clip typed schema + loader + lossless conversion** `risk:medium (geometry conversion)` `depends:[]`
  > After this: cargo test carica Agumon clip.ron come Clip tipizzato e asserisce equivalenza geometrica lossless con agumon_atlas.json (frame_size/columns/rows/total_frames + ogni range clip).

- [ ] **S03: Validator §L completo (contract and cross-asset)** `risk:medium (validator logic)` `depends:[S01,S02]`
  > After this: tests/anim_fsm_validation.rs verde — Agumon valido passa l'intero §L; un fixture rotto per ogni check fa fallire le boot con DataError che nomina file+check; reachability/cancel solo warning.

- [ ] **S04: Hot-reload and degenerate stub 5 non-Agumon** `risk:low (bevy-native)` `depends:[S01,S02]`
  > After this: cargo run --features windowed con Agumon, edit a caldo di clip.ron/anim_graph.ron ricarica l'asset senza crash né stato sporco (UAT).

## Boundary Map

### S01 → S03

Produces:
- Tipo `AnimGraph` (nodi: `NodeId → Node{frames:(u32,u32), on_enter:Vec<Command>, modifier:Option<Modifier>, reverse:Option<bool>}`; `transitions: Vec<Edge{from,to,when:Predicate,priority:u8}>`; `entry: NodeId`; `clip: String`).
- Enum chiusi: `Command` (§C 6 verbi + §C2 7 verbi), `Predicate` (§D), `ParamRef` (§S), `TargetShape` (§C3).
- `RonAssetPlugin::<AnimGraph>` registrato nel plugin asset, handle tracker stile `src/data/mod.rs`.

Consumes:
- nothing (first slice)

### S02 → S03

Produces:
- Tipo `Clip` (`meta{frame_size,columns,rows,total_frames}`, `clips: Map<String, ClipRange{start,end,fps,loop}>`); loader-side defaults fps/loop/texture_path.
- Agumon `assets/digimon/agumon/clip.ron` generato lossless da `agumon_atlas.json`.
- `RonAssetPlugin::<Clip>` registrato; lifecycle/tracker condiviso with S01.

Consumes:
- nothing (first slice)

### S01 → S04

Produces:
- `AnimGraph` loader hot-reloadabile (RonAssetPlugin + AssetServer file-watch).

Consumes:
- `AnimGraph` type + loader (S01).

### S02 → S04

Produces:
- `Clip` loader hot-reloadabile.

Consumes:
- `Clip` type + loader (S02).

### S03 → (milestone exit)

Produces:
- `tests/anim_fsm_validation.rs` — validator §L completo come contract test; `DataError` boot-fail su fixture rotti.

Consumes:
- `AnimGraph` (S01) + `Clip` (S02) per la validazione cross-asset.
