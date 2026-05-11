# §2.6 — Kernel suspend/resume — coroutine-style skill execution

**Problema:** QTE è solo un caso particolare di una necessità più generale. Una skill può aver bisogno di:

- **Input giocatore mid-skill** (QTE, scelta target a metà flow, picker di rami)
- **Logica custom blueprint** che vuole calcolare/decidere/animare e poi tornare al kernel con un risultato
- **Animation gate** che blocca il flow finché il VFX/animazione del cast non è arrivato a un frame chiave (es. "applica danno sul frame 8 dell'attacco")
- **External event** (network, scripting hook in futuro)

Senza un meccanismo unificato, ogni caso diventa un branch ad-hoc nel resolver. Con un meccanismo unificato, sono tutti istanze dello stesso pattern: **il kernel sospende, qualcun altro lavora, il risultato torna al kernel, il flow riprende**.

**Decisione architetturale:** il resolver del flow è una **state machine pausabile**. A ogni step l'esito è uno di:

- `Advance` — applica `KernelEffect` accumulati, vai al prossimo step
- `Done` — flow terminato (commit cost refund/surcharge, emetti `SkillResolved`)
- `Suspend { reason, cursor }` — congela lo stato kernel, persiste il `cursor`, attende `SkillYieldResolved`

**A — Tipi base:**

```rust
pub struct SkillCursor {
    skill_id: SkillId,
    flow_path: SmallVec<[usize; 4]>,  // [step_idx, branch_then_idx, …] per flow annidati
    locals: SkillLocals,              // last_struck, last_killed, accumulated damage, ecc.
}

pub enum YieldReason {
    QuickTimeEvent { kind: QteKind, window_ms: u32, headless_default: QteOutcome },
    BlueprintHook { hook_name: String, owner: BlueprintId, payload: HookPayload },
    AnimationGate { gate_id: String, timeout_ms: u32 },     // facoltativo M017
    PlayerChoice { kind: ChoiceKind, options: Vec<ChoiceOption> },
    CascadeComplete,  // §2.8: la skill chiede di vedere lo stato post-cascade prima di proseguire
}

pub struct SkillYieldResolved {
    cursor_id: CursorId,
    outcome: YieldOutcome,  // QteOutcome | HookOutcome { effects: Vec<KernelEffect> } | Choice(idx) | …
}
```

**B — Phase nel `CombatPhase`:**

Aggiunge una variant: `CombatPhase::AwaitingSkillYield { cursor_id, reason }`. Mentre il kernel è in questa fase:

- `advance_turn_system` non gira (turn order congelato)
- `resolve_action_system` è in stato "parking" — non scrive su `CombatState` finché non riceve `SkillYieldResolved`
- `enemy_ai`, `status_effect tick`, `follow_up` — tutti gated su `CombatPhase != AwaitingSkillYield`
- `CombatEvent` bus continua a poter **emettere** notifiche (UI le legge), ma il kernel non consuma intent

**C — Contratto di sospensione (chi può fare cosa):**

| Attore | Può durante suspend? |
|---|---|
| UI / windowed | Legge stato, mostra prompt QTE / picker / animazione, emette `SkillYieldResolved` |
| Blueprint plugin (owner skill) | Riceve `BlueprintHook` payload, esegue logica Rust, accumula `KernelEffect`, emette `SkillYieldResolved { outcome: HookOutcome { effects } }` |
| Test scaffold | Auto-risolve con default scriptato (es. `qte_resolver = MockAlwaysSuccess`) |
| Altri systems kernel | **Niente.** Read-only su `CombatState`. Niente damage/status/turn tick |

**D — Default headless (per AI test e replay):**

Ogni `YieldReason` ha un fallback deterministico:

- `QuickTimeEvent` → `headless_default: Success|Fail` (campo del flow RON)
- `BlueprintHook` → il plugin owner ha un'implementazione headless dichiarata (no input richiesto); contract test al boot verifica che ogni hook abbia implementazione registrata
- `AnimationGate` → timeout → auto-resolve in headless è istantaneo
- `PlayerChoice` → policy in `unit_ai.ron` (es. "scegli sempre opzione 0" o "scegli random seedato")
- `CascadeComplete` → kernel drena la coda effetti + reazioni, poi richiama `execute()` con stato post-cascade. Non richiede input esterno; è una sospensione "interna" al loop kernel. Nessun resolver UI/blueprint coinvolto.

Nessuno yield può bloccare i test headless: assenza di resolver → applicazione default → log di warning + continuazione.

**E — Determinismo & replay:**

- `cursor_id` è seedato sul `(combat_seed, turn_idx, skill_id, action_uid)` → stabile su replay
- `SkillYieldResolved` viene loggato nel `jsonl_logger` con `cursor_id` + `outcome` → replay-from-log riapplica gli stessi esiti
- Hook effects accumulati arrivano come `Vec<KernelEffect>` (stesso bus di §2.3) → il kernel ne conosce solo le variant chiuse

**F — Limiti v1 (M017) per non esplodere il modello:**

- **Niente nested yield** in M017: un blueprint hook che sospende → non può a sua volta sospendere. Stack depth = 1. Si rilassa solo se emerge necessità reale.
- **Niente timeout reale wall-clock**: i timeout sono in `ticks` logici, non secondi → mantiene determinismo. Per UI windowed: il timeout perceived viene gestito a livello presentation (animazione del prompt), il kernel vede solo "tick scaduto, applica default".
- **Niente cancel mid-yield**: una volta sospesi, si esce solo via resolve (anche con default). Niente "annulla la skill". Se serve in futuro → variant esplicita `YieldOutcome::Abort`.

**G — Costo (impatta §5):**

- **S03d "Kernel suspend/resume"** isola: enum `YieldReason`/`YieldOutcome`, `SkillCursor`, fase `AwaitingSkillYield`, gating systems, JSONL logger del yield event. Risk: high (tocca il cuore del turn loop).
- Test integration headless deve dimostrare: (a) replay-stability con yield seedato; (b) gating systems durante suspend; (c) default fallback funzionante.

Per un esempio end-to-end di skill multi-fase con QTE + animation gate + emit di effetti vedi **§2.9 (worked example)**.

**H — Cosa NON entra in M017 (out of scope §2.6):**
- Nested yield (suspend dentro suspend)
- Cancel mid-yield
- Yield wall-clock based (al posto di tick logici)
- Suspend cross-turn (la skill termina sempre nello stesso turno; multi-turn → Phase, non Suspend)
