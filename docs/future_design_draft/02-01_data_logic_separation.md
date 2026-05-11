# §2.1 — Separazione dati/logica (RON = numeri)

**Stato attuale:** `assets/data/skills.ron` mischia numeri (damage, sp_cost, ult_cost) e routing logico (`custom_signals` con owner+signal+payload). Il routing è già *dichiarativo* (no codice nel ron), ma la *forma* dei signal è ambigua tra "design" e "implementazione".

**Target — tre layer disaccoppiati:**

| Layer | File | Contenuto | Sa di chi? |
|---|---|---|---|
| Dati skill | `units.ron`, `skills.ron`, `party.ron` | **Solo numeri e identificatori** (damage, sp_cost, target shape, skill_id) | Nessuno |
| Trigger gameplay | `assets/data/signal_bindings.ron` | `{ skill: "thunder_loop", signal: "build_exploit" }` — glue layer skill ↔ signal | Conosce skill_id + signal name |
| Behaviour | blueprint Rust (`blueprints/<owner>/`) | Definisce signal + handler logic | Conosce solo il signal name |

**Perché `custom_signals` esce da `skills.ron`:** la skill non deve sapere chi ascolta; il signal non deve sapere chi triggera. `signal_bindings.ron` è glue esterno. Skill può vivere senza signal, signal può esistere senza skill specifica. Mettere `custom_signals` in `skills.ron` accoppia skill al sistema event → male.

**Regola di routing:** il ron descrive *cosa* (numeri, target shape, skill_id), `signal_bindings.ron` descrive *quale signal* parte *da quale skill*, il blueprint Rust descrive *come* il signal viene gestito. Niente flag di feature, niente ramificazioni `if name == "Tentomon"`.

**Test di consistenza proposto:** un test che fallisce se in `signal_bindings.ron` compare un signal il cui owner/skill non è registrato (analogo a `transitions_for_action_checked`; sollevarlo come gate al boot di `DataPlugin`).

> **Nota naming (no overlap):** `signal_bindings.ron` (gameplay glue, path `assets/data/`) governa **cosa** triggera quale signal kernel. `clipmontage.ron` (animation presentation, path `assets/digimon/<name>/`) governa **come** si vede. File diversi, scopi diversi, layer diversi. Boundary stretto in §2.2.
