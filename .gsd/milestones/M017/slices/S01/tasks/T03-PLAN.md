---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: RON migration (assets/data/skills.ron + units.ron)

Sostituire tutti i status id legacy in assets/data/skills.ron (11 occorrenze) e assets/data/units.ron (3 occorrenze) con i 5 id canon. Mappa di traduzione di lavoro: Burn->Heated, Freeze->Chilled, Shock->Paralyzed, DeepFreeze->Slowed. Blessed entra solo dove un buff offensivo è già modellato (probabile zero match attuali). Verifica che ogni occorrenza sia status id (non skill name come 'baby_flame'). Niente cambio di durate/numeri.

## Inputs

- `Validator id list da T02`

## Expected Output

- `Tutti gli status id RON canon-only`

## Verification

cargo run --bin combat_cli (smoke) carica i RON senza loader error. Test di parsing RON in T05 confermano.
