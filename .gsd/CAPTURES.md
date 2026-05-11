# Captures

### CAP-749a38e2
**Text:** bisogna evitare enum per i signals dei digimons, bisogna rendere meno statica la cosa. valuta tecniche diverse per "registrare" dinamicamente i signals e tutto ciò che è appartenente a un digimon. dovrebbe essere plug-in. se non voglio più un digimon=toglo solo 1 file e rimuovo il plugin
**Captured:** 2026-05-09T10:44:20.183Z
**Status:** resolved
**Classification:** replan
**Resolution:** Replan current S02 work before execution so Dorumon signal ownership can evaluate dynamic/plugin registration instead of proceeding with the static enum-only custom signal design.
**Rationale:** The capture directly conflicts with the current S02 must-have requiring Dorumon-specific enum variants in `src/data/skills_ron.rs`; remaining incomplete tasks may need rewriting around a less static per-Digimon signal/plugin boundary.
**Resolved:** 2026-05-09T13:52:18.887Z
**Milestone:** M016
**Executed:** 2026-05-09T13:52:41.718Z
### CAP-92aab67d
**Text:** batteryloop e gli altri non dovrebbero essere qualcosa di condiviso, ma specifico del digimon... non mi piace che siano direttamente sotto "combat".
**Captured:** 2026-05-09T12:58:56.597Z
**Status:** resolved
**Classification:** replan
**Resolution:** Replan current S02 and remaining M016 blueprint work to account for per-Digimon mechanic ownership and avoid treating loops like BatteryLoop/PredatorLoop as shared `combat` modules without review.
**Rationale:** The capture challenges the shared mechanic-module layout that S02 currently consumes (`src/combat/predator_loop.rs`) and that S01 established for BatteryLoop, so the current slice shape may need adjustment rather than a small additive task.
**Resolved:** 2026-05-09T13:52:18.887Z
**Milestone:** M016
**Executed:** 2026-05-09T13:52:41.719Z
