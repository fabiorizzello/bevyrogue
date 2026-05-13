# Tier 3 — Q ancora aperte (continue cross-machine)

Stato al `2026-05-13`. Branch: `milestone/M017`. Fonte: `.gsd/quick/1-cattura-questo-lavoro/BACKLOG.md` (gitignored, non viaggia su altri PC — questo file lo sostituisce).

## Tier 3 status

| # | Task | Status | File target |
|---|------|--------|-------------|
| Q7 | Template `blueprints/{name}/` + `Plugin` trait + Agumon canary | **DONE** (commit `5d8a23f`) | `src/combat/blueprints/agumon/{mod.rs, signals.rs, identity.rs}` |
| Q8 | Migrare `twin_core.rs` → `blueprints/agumon/identity.rs` | **DONE** (commit `5d8a23f`) | idem |
| Q9-patamon | Migrare `holy_support.rs` (254 LOC) → `blueprints/patamon/identity.rs` | **OPEN** | `src/combat/holy_support.rs` |
| Q9-tentomon | Migrare `battery_loop.rs` (261 LOC) → `blueprints/tentomon/identity.rs` | **OPEN — preferibilmente S0 di M026** | `src/combat/battery_loop.rs` |
| Q9-renamon | Migrare `precision_mind_game.rs` (211 LOC) → `blueprints/renamon/identity.rs` | **OPEN — preferibilmente S0 di M027** | `src/combat/precision_mind_game.rs` |
| Q9-dorumon | Migrare `predator_loop.rs` (510 LOC) → `blueprints/dorumon/identity.rs` | **OPEN** | `src/combat/predator_loop.rs` |
| Q9-gabumon | Gabumon **non ha identity passive** in `src/combat/` (manca da BACKLOG); `blueprints/gabumon.rs` esiste solo per signal dispatch | **N/A** o trivial | `src/combat/blueprints/gabumon.rs` |
| Q10 | `App::add_plugins((AgumonPlugin, PatamonPlugin, ...))` finale + rimozione `pub mod twin_core` shim e simili | **OPEN — quando tutti i Q9 sono in** | `src/combat/kernel.rs` registration site |

## Raccomandazione stagger

- **Q9 just-in-time per milestone feature:**
  - Q9-tentomon = S0 di **M026** (AoE Tentomon — atterra in `battery_loop`)
  - Q9-renamon = S0 di **M027** (time-manip Renamon — atterra in `precision_mind_game`)
  - Q9-patamon / Q9-dorumon: milestone cleanup dedicato, o quando una feature li tocca
- **Q10 a chiusura** quando tutti e 4 i Q9 sono in.

Razionale: Q7+Q8 hanno già pagato il setup-cost e validato la seam con Agumon (147 lib tests + 532 integration verdi). Migrare codice "stabile" che non sta per essere toccato sarebbe debt anticipata — meglio agganciare la migrazione alla milestone che già modifica quel file.

## Pattern di riferimento (Agumon canary)

Per replicare la migrazione su un altro digimon:

1. `mkdir src/combat/blueprints/<name>/`
2. Spostare il file identity (es. `holy_support.rs`) in `blueprints/<name>/identity.rs`
3. Spostare/creare `blueprints/<name>/{mod.rs, signals.rs}` (signals = custom RON dispatch, già presente nei `blueprints/<name>.rs` flat correnti)
4. Definire `<Name>Plugin` con `impl Plugin for <Name>Plugin { fn build(&self, app) { … } }` che registra state resource + observe hook + relevant systems
5. `register_combat_kernel_runtime` (in `kernel.rs`) → `app.add_plugins(<Name>Plugin)` invece di registrare inline
6. Lasciare backward-compat shim in `combat/mod.rs`: `pub use blueprints::<name>::identity as <old_module_name>;` per non rompere ~N import esistenti
7. Verifica: `cargo test --workspace`

Vedi commit `9340acf` (refactor Agumon) e merge `5d8a23f` per il diff completo.

## DAG dipendenze

- M018 (Time-manip split + TargetShape) → non tocca identities, può procedere ortogonalmente
- M026 (Tentomon AoE) → dipende M018 → Q9-tentomon as S0
- M027 (Renamon time-manip) → dipende M018 → Q9-renamon as S0
- Q10 → dipende tutti Q9
