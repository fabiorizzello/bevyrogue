# Combat Design Draft (future work)

**Status:** draft per validazione, non ancora milestone planning. Estende `combat_current.md` (canon post-M016).

## Indice

| # | File | Sezione |
|---|---|---|
| 1 | [01_goal.md](01_goal.md) | Goal |
| 2 | — | **Constraint architetturali (da validare prima del planning)** |
| 2.1 | [02-01_data_logic_separation.md](02-01_data_logic_separation.md) | Separazione dati/logica (RON = numeri) |
| 2.2 | [02-02_animation_manifest.md](02-02_animation_manifest.md) | Animation manifest (.ron per atlas) — modello UE5 a 2 asset |
| 2.2b | [02-02b_animation_fsm.md](02-02b_animation_fsm.md) | Animation FSM — clipmontage come grafo orchestratore (amendment §2.2: unlock/branching/QTE/cancel) |
| 2.2c | [02-02c_dematerialize_pattern.md](02-02c_dematerialize_pattern.md) | Dematerialize/Rematerialize — shader bypass del movement blending (default uniforme per tutte le skill close-range) |
| 2.3 | [02-03_blueprint_plugin.md](02-03_blueprint_plugin.md) | Blueprint plugin extension (kernel estensibile, blueprint isolati) |
| 2.4 | [02-04_strong_typing.md](02-04_strong_typing.md) | Extension-first + strong-typing (kernel chiuso, blueprint aperti) |
| 2.5 | [02-05_tunable_catalog.md](02-05_tunable_catalog.md) | Tunable data catalog (editor-ready) |
| 2.6 | [02-06_kernel_suspend_resume.md](02-06_kernel_suspend_resume.md) | Kernel suspend/resume — coroutine-style skill execution |
| 2.7 | [02-07_skill_as_plugin.md](02-07_skill_as_plugin.md) | Skill-as-Plugin — kernel chiuso senza catalogo di skill primitives |
| 2.8 | [02-08_effect_cascade.md](02-08_effect_cascade.md) | Kernel effect cascade — resolution & reactive chains |
| 3 | [03_run_loop_cli.md](03_run_loop_cli.md) | Run-loop CLI (gameplay scope) |
| 4 | [04_enemy_roster.md](04_enemy_roster.md) | Enemy roster |
| 5 | [05_slices.md](05_slices.md) | Slice candidate (ordering by risk) — **allineato §8 minimal** |
| 6 | [06_out_of_scope.md](06_out_of_scope.md) | Out of scope esplicito |
| 7 | [07_definition_of_done.md](07_definition_of_done.md) | Definition of done |
| 8 | [08_roster_minimal.md](08_roster_minimal.md) | **Roster minimal (canon).** 6 Rookie, kit uniforme (Basic/Skill/Ult/Passive), 1 modifier-firma per Digimon (eccetto Patamon). Niente skill-tree, niente varianti — deferred. |
| 9 | [09_ui_surface.md](09_ui_surface.md) | UI surface — **allineato §8 minimal** |
| 10 | [digimon/](digimon/) | **Stress test per-Digimon** (idle→skill/attack/ult→idle): valida §2.2b FSM su atlas reali, raccoglie gap prima di M017. Attivo: `agumon/`. |

## Nota sulla riduzione di scope (2026-05-11)

I doc precedenti `08_skill_designs.md`, `10_full_kit_plan.md`, `11_roster_design_v2.md` sono stati **rimossi**: troppi modifier reattivi, skill-tree, status set extension, kit eterogeneo, passive multilivello. La direzione corrente è **all'osso**: vedi §8 (`08_roster_minimal.md`) come unica fonte di verità per identità e kit dei 6 Rookie.

Inoltre rimosso `02-09_worked_example.md` (Pepper Breath full-featured con 2 unlock skill-tree `super_charge` + `triple_hit`): faceva riferimento a concetti deferred dal pivot minimal (skill-tree, variant). Quando servirà un reference end-to-end del pattern FSM, lo si riscrive aderente al kit §8.
