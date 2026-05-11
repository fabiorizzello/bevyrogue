# Agumon — Stress test FSM AnimGraph

Stress test del design canon §8 contro l'architettura §2.2b Animation FSM. Obiettivo: scovare contraddizioni timing / vocabolario / kernel-coupling **prima** di M017.

## Index

| File | Scope | Output |
|---|---|---|
| `00_identity.md` | Identità, atlas mapping, kit shape, timing convention | Baseline + drift legacy |
| `01_basic_claw_strike.md` | Basic 3-nodi `Windup→Strike→Recovery` | 4 gap (params, chance, headless drop, ult-charge timing) |
| `02_skill_pepper_breath.md` | Heavy 4-nodi `Inhale→Wind→Spit→Recovery` | 3 gap (tough_break, stacks param, ordering) |
| `03_ult_nova_blast.md` | Ult 4-nodi con edge reattivo `OnKill→ReactiveDetonate` + QTE | 5 gap nuovi (event payload, multi-target, frame budget, QTE window) |
| `04_passive_twin_core_fire.md` | Listener passive + aggregato 12 gap cross-file | Top-3 da risolvere: G1, G5, G9 |

## Reading order

1. `00_identity.md` → setup
2. `01` → `02` → `03` → ordine crescente complessità
3. `04_passive_twin_core_fire.md` → listener + § aggregato gap

## Convenzione timing

- **Frame counter logico** = autoritativo (per §2.2b §G headless determinism)
- **ms @12fps** = annotazione lettura (1 frame ≈ 83ms)
- Stretch via `Hold { extra_frames }`, **mai cambiare ms**

## Atlas

`assets/digimon/agumon_atlas.json` v1 — 84 frames, 8 animazioni nominali. Mapping kit:

- Basic = `attack` (0–8, 9f)
- Heavy = `heavy_attack` (23–36, 14f)
- Ult = `skill` (50–66, 17f)
- Idle loop = `idle` (44–49, 6f)
