# Decisions Register

<!-- Append-only. Never edit or remove existing rows.
     To reverse a decision, add a new row that supersedes it.
     Read this file at the start of any planning or research phase. -->

| # | When | Scope | Decision | Choice | Rationale | Revisable? | Made By |
|---|------|-------|----------|--------|-----------|------------|---------|
| D001 | M001 planning | architecture | Animation FSM module ownership | One cohesive animation module boundary | The user explicitly wanted the whole animation FSM area from schema and orchestration through future runtime to reside in a single conceptual module boundary, avoiding scattered ownership across data/combat/runtime layers. | Yes | human |
| D002 | M001 planning | architecture | Animation validator coupling boundary | Adapter based cross-asset validation | This keeps validation strong against real project data while preserving a generic animation motore and preventing direct coupling to Digimon/gameplay internals inside core animation code. | Yes | collaborative |
| D003 | M001 planning | planning | Use of prior M022 plan | Adapt M022 intent to the current repo rather than copying it literally | The user clarified that M022 came from MILESTONE_PORTFOLIO and should define what needs implementing, but not impose rigid implementation rules when current architecture can be improved. | Yes | human |
| D004 | M001 planning | operability | Animation asset failure policy | Strict on boot, resilient on reload | Boot-time invalid assets should fail fast with typed diagnostics so bad data does not reach runtime, while hot-reload authoring should preserve the last valid state and log clearly instead of crashing. | Yes | collaborative |
