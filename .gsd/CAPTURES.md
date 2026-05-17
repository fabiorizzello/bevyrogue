# Captures

### CAP-8d133d1a
**Text:** vedo un sacco di file relativi a identità custom dei digimon ma sotto combat/ battery_loop, precision_mind_game(è ancora usato?)? tieni conto che non dobbiamo mantenerci retrocompatibili o altro. voglio che dal kernel spariscano le logiche/strutture dati custom dei specifici digimon. in più i test che non servono realmente si possono togliere. è normale avere tutti quei test per il nostro numeor di LOC del progetto effettivo? Fai un passaggio di prune/remove orphan code/obsoleto. magari tramite un ultimo slice a fine m021
**Captured:** 2026-05-17T06:50:03.872Z
**Status:** resolved
**Classification:** defer
**Resolution:** Defer to a final M021 cleanup slice for pruning Digimon-specific kernel logic, orphan code, and obsolete tests.
**Rationale:** This is important follow-up work, but it belongs in a later cleanup slice rather than changing the current S12 scope.
**Resolved:** 2026-05-17T08:18:14Z
**Milestone:** M021

### CAP-af4db4ca
**Text:** anche enemy_counterplay - ogni enemy avrà il suo counterplay, non bisogna inserire la logica nel kernel, se non primitive
**Captured:** 2026-05-17T06:51:21.060Z
**Status:** resolved
**Classification:** defer
**Resolution:** Defer as a design constraint for future enemy implementations: keep counterplay logic in per-enemy modules/blueprints, not in kernel code.
**Rationale:** This is a reusable architectural note, but it does not require immediate action in the current slice.
**Resolved:** 2026-05-17T08:18:14Z
**Milestone:** M021
