---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Effect::ApplyStatus RON schema + validator

Aggiornare src/data/skills_ron.rs: Effect::ApplyStatus accetta i 5 id canon ('heated', 'chilled', 'paralyzed', 'slowed', 'blessed'). Validator a load-time rigetta id legacy ('burn_v0', 'freeze_v0', etc.) con messaggio chiaro che indica i 5 id validi. Eventuali costanti id collegate riscritte.

## Inputs

- `StatusKind nuovo da T01`

## Expected Output

- `Loader RON status canon-only`
- `Errore esplicito su id legacy`

## Verification

cargo check verde. Test di parsing RON esistenti continuano (saranno aggiornati ai nuovi id in T03/T05).
