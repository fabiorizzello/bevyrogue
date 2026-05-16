# Decisions Register

<!-- Append-only. Never edit or remove existing rows.
     To reverse a decision, add a new row that supersedes it.
     Read this file at the start of any planning or research phase. -->

| # | When | Scope | Decision | Choice | Rationale | Revisable? | Made By |
|---|------|-------|----------|--------|-----------|------------|---------|
| D001 | M021/S07/T04 | combat | Emit `BlockReactionTriggered` whenever a one-shot passive mitigation modifier is consumed in the incoming-damage pipeline, not only when Tentomon's reactive block proc succeeds. | Use the shared event as the canonical diagnostic surface for any pre-damage passive mitigation, with Tentomon's block reaction and generic passive damage modifiers both funneled through the same event. | This keeps the damage pipeline and the Tentomon passive aligned, preserves the existing generic block-reaction test, and gives one observable seam for future passive mitigation effects. | Yes | agent |
| D002 |  | architecture | How TwinCore transitions route through CombatKernelTransition after removing the digimon-specific variant | Replace `CombatKernelTransition::TwinCore(TwinCoreTransition)` with `CombatKernelTransition::Blueprint { owner: "twin_core", name: "<signal_name>", payload: SignalPayload::Amount(amount) }`. The `apply_twin_core_transitions_system` decodes Blueprint events where owner=="twin_core" and maps name to the corresponding TwinCoreSignal internally within the twin_core module. | M021 CONTEXT M5 mandates removal of 5 digimon-specific variants from CombatKernelTransition, replaced by a generic Blueprint variant. TwinCore is shared between Agumon and Gabumon but is still a blueprint-level mechanic, not a kernel primitive. Routing through Blueprint { owner: "twin_core" } keeps the kernel generic while preserving full observability via JSONL (same event shape as passive triggers from S07). | Yes | agent |
