# §2.8 — Kernel effect cascade — resolution & reactive chains

**Problema:** la skill emette `KernelEffect::Damage` + `KernelEffect::ApplyStatus { burn }`. Un *alleato* ha passiva "quando viene applicato burn, applico shock". La passiva è un'altra logica blueprint. Chi orchestra? Quando? In che ordine?

**Modello:** kernel-owned cascade come estensione del `CombatEvent` bus. **Ogni `KernelEffect` applicato emette un `CombatEvent`; ogni blueprint può sottoscriverlo ed emettere nuovi `KernelEffect`; il kernel drena finché la coda è vuota.**

**Scope:** la cascade è generica — **non** solo "attacco-dopo-evento". Ogni blueprint dichiara *cosa ascolta* (CombatEventKind) e *cosa emette* (qualsiasi `KernelEffect`). I pattern coperti:

| Pattern reattivo | Trigger event | Effetto emesso |
|---|---|---|
| Vendetta (follow-up attack) | `Damaged{target=ally}` | `Damage` o spawn `FollowUpAction` (vedi §E) |
| Counter-status | `StatusApplied{burn,target=ally}` | `ApplyStatus{shock}` sull'attaccante |
| Cleanse passivo | `StatusApplied{poison,target=ally}` | `RemoveStatus{poison}` |
| Charge gain on kill | `UnitKilled{by_team=mine}` | `GrantUltCharge{amount}` |
| Buff aura on entry | `UnitSpawned{side=ally}` | `ApplyStatus{atk_up,duration:permanent}` |
| Signal broadcast | `StatusApplied{exposed,target=enemy}` | `EmitSignal("exposed_marker")` (consumato da skill custom in §2.7) |
| Defensive interrupt (M018+) | `IncomingDamage{target=ally}` | `PreApplyHook("reduce")` — non in scope M017 |

Il kernel **non enumera** questi pattern. Espone solo: bus eventi + queue effetti + drain loop + cap. Ogni Digimon plugin è libero di reagire a *qualsiasi* `CombatEvent` ed emettere *qualsiasi* `KernelEffect`. La logica "cosa fa Tentomon quando un alleato viene bruciato" vive **interamente** in `blueprints/tentomon/reactions.rs`, non in `combat/*.rs`.

## A — Il ciclo (drain loop)

```
skill.execute() → emette N KernelEffect via ctx.emit() → ritorna Done/Suspend
                                  ↓
                  effects_queue.push(N effetti, FIFO)
                                  ↓
          ┌──────── kernel drain loop ────────┐
          │  pop effetto                       │
          │   ├─ apply (muta state)            │
          │   ├─ emit CombatEvent (StatusApplied/Damaged/...) │
          │   └─ blueprint subscribers reagiscono → push nuovi effetti  │
          │  loop finché queue empty           │
          └────────────────────────────────────┘
                                  ↓
        kernel ritorna controllo a skill (se Suspend) o passa al turn successivo
```

Tre invarianti:
1. La skill **non** vede applicazioni intermedie durante un singolo `execute()`: ritorna il batch e il kernel lo processa atomicamente.
2. Le reazioni dei blueprint **estendono** il batch corrente (push in coda), non aprono un batch separato. Tutta la cascade è un'unica drain run.
3. Se la skill ha bisogno di leggere stato post-cascade (es. "se il damage ha killato → branch"), suspende esplicitamente con `Suspend(CascadeComplete)` o splitta in due `execute` step. Default: la skill emette tutto in un colpo e il kernel risolve.

## B — Subscribers (chi reagisce)

Pattern coerente con §2.3 (blueprint plugin extension):

```rust
// blueprints/<owner>/<name>.rs
fn register_reactions(app: &mut App) {
    app.add_systems(
        CombatSchedule::ReactToEffect,
        on_burn_applied_shock_back.run_if(blueprint_owner_alive),
    );
}

fn on_burn_applied_shock_back(
    mut events: MessageReader<CombatEvent>,
    mut effects: ResMut<KernelEffectQueue>,
    query: Query<(Entity, &MyBlueprintTag, &TeamSide)>,
) {
    for ev in events.read() {
        if let CombatEventKind::StatusApplied { kind: StatusKind::Burn, target, .. } = ev.kind {
            for (owner, _, side) in &query {
                if side.is_ally_of(target) {
                    effects.push(KernelEffect::ApplyStatus {
                        target,
                        status: "shock",
                        stacks: 1,
                        source: owner,
                    });
                }
            }
        }
    }
}
```

Il kernel non ha listener interni hard-coded — espone solo `KernelEffectQueue` come resource scriviabile + `CombatEvent` come bus. I subscribers vivono nei plugin dei blueprint, esattamente come la state machine privata di Tentomon.

## C — Determinismo (la parte critica)

Una cascade reactive senza ordine ben definito = replay rotto + AI test flaky. Tre garanzie:

1. **FIFO sulla queue**: ordine di inserimento = ordine di applicazione. Niente priorità, niente "interrupt".
2. **Stable subscriber order**: quando più blueprint reagiscono allo stesso `CombatEvent`, ordine = `(owner_entity_id_ascending, system_registration_order)`. Bevy schedule lo garantisce se non si usano `.before/.after` arbitrari.
3. **Reazione = stesso turn, stesso batch**: la cascade non "salta a un turno futuro". Tutto avviene prima che il kernel ceda controllo.

## C-bis — Loop prevention by design

**Niente cap "magico" come safety net globale.** Un cap nasconde il problema: una cascade runaway è un bug di design (passive che si re-triggerano) e silenziarlo con abort+diagnostic = fix mancato. Approccio: rendere **strutturalmente impossibile** il loop infinito, non assorbirlo.

Due meccanismi combinati. Sono **obbligatori** per ogni reaction registrata (contract test al boot).

**1. Idempotency scope (dichiarato nel manifest del subscriber)**

Ogni reaction dichiara la propria *scope di unicità* — il kernel traccia internamente un set di `(reaction_id, scope_key)` già fired e skippa i duplicati.

```rust
// blueprints/<owner>/reactions.rs
ReactionManifest {
    id: "tentomon::shock_on_ally_burn",
    listens_to: CombatEventKind::StatusApplied { kind: Some(StatusKind::Burn) },
    idempotency: IdempotencyScope::OncePerCause { cause_kinds: [EventCauseKind::Action] },
    emits: [KernelEffectKind::ApplyStatus],
}
```

Valori di `IdempotencyScope`:

| Scope | Significato | Esempio d'uso |
|---|---|---|
| `OncePerAction` | La reaction fira max 1 volta per ogni `CauseId` di tipo `Action`. Ri-firing nello stesso batch sullo stesso causation tree = no-op. | "shock on burn": se nello stesso attacco vengono applicati 3 burn, shock back fira **una volta sola**. Loop A↔B↔A: A fira al burn iniziale, push shock; B fira al primo shock, push burn (con cause = quel ramo); A non rifira perché già marcata su quel CauseId. Stop naturale. |
| `OncePerCause { cause_kinds }` | Variante: fira max 1 volta per cause di una *classe* (Action / FollowUp / Reaction). Equivale a `OncePerAction` quando `cause_kinds=[Action]`. | Più espressivo: passiva che ascolta solo cause `Action` non ricorre su `Reaction`. |
| `OncePerTarget` | Max 1 volta per target colpito, per action. | "rappresaglia su nemico": un nemico colpisce 3 ally, l'aura fira 1 volta sull'attaccante, non 3. |
| `EveryTime` | Niente guard, fira sempre. Usare solo per effetti che si auto-limitano (es. status non rifresca durata) — manifest deve dichiarare la motivazione. |

Lo scope chiave: **`OncePerAction` rompe i loop A↔B↔A by construction**. La fase A→B→A non può chiudersi perché A è già marcata.

**2. Event provenance taxonomy (cause kind)**

Ogni `KernelEffect` propaga un `cause: CauseChain` che il kernel popola automaticamente. Il subscriber filtra by class:

```rust
enum EventCauseKind {
    Action,    // skill primaria scelta dal player/AI nel turno corrente
    FollowUp,  // azione reattiva spawned da reaction → KernelEffect::EnqueueAction (Q4=b)
    Reaction,  // KernelEffect emesso da subscriber, non da skill
    System,    // tick passivo (status duration, turn start, ecc.)
}

struct CauseChain {
    id: CauseId,                  // u64 stabile per la radice (action_uid)
    kind: EventCauseKind,
    parent: Option<CauseId>,      // permette tracing della reaction chain
    depth: u8,                    // info-only, NON usata come cap
}
```

Il manifest di una reaction può filtrare in dichiarativo:

```rust
ReactionManifest {
    id: "agumon::vendetta_on_ally_attacked",
    listens_to: CombatEventKind::Damaged { target_filter: TargetFilter::Ally },
    cause_filter: CauseFilter::Only(&[EventCauseKind::Action]),  // ← solo attacchi diretti
    idempotency: IdempotencyScope::OncePerAction,
    emits: [KernelEffectKind::EnqueueAction],
}
```

Conseguenza: la vendetta di Agumon **non** fira su un `Damaged` causato da una `Reaction` o `FollowUp`. Il loop "vendetta scatena vendetta" non può iniziare perché la 2ª iterazione cambia `cause.kind` (`Action → FollowUp → Reaction`).

`cause_filter` è **obbligatorio**: una reaction senza `cause_filter` dichiarato non compila (contract test). Forza il designer a pensare alla provenance.

**3. Combinazione = niente loop infiniti possibili**

| Scenario | Senza guard | Con guard |
|---|---|---|
| Passiva A "on burn → shock"; passiva B "on shock → burn" | ∞ | A fira (OncePerAction marca A); pusha shock; B fira (OncePerAction marca B); pusha burn; A vede burn ma `cause` punta allo stesso CauseId iniziale → A skippata. Cascade chiude in 2 livelli. |
| Vendetta su attacco alleato, ma non su follow-up | Follow-up triggera vendetta che triggera follow-up = ∞ | `cause_filter: [Action]` blocca: il follow-up ha `kind=FollowUp`, vendetta lo ignora. |
| AoE colpisce 4 nemici, ognuno con counter-attack | 4×4 explosion | `OncePerTarget` su counter-attack: ogni nemico contro-attacca 1 volta. 4 counter, non 16. |

Nessun cap, nessun abort silenzioso. Loop prevention vive **nella dichiarazione della reaction**, kernel-side la enforce-a tramite `(reaction_id, cause_id)` set per drain run.

**4. Safety net diagnostico (solo `EveryTime` malformata)**

- Contract test sintetico al boot prova ogni reaction contro fixture di eventi → fail CI se loop emerge < 50 iterazioni.
- Runtime debug: cap diagnostico alto (1024 effetti, log-only, **non aborta**) stampa la cause chain. Strumento di sviluppo, **non** semantica di produzione.
- Runtime release: nessun cap. Il modello strutturale garantisce convergenza by design.

## D — Cosa NON entra (limiti M017)

- **Passive yield**: un subscriber **non può** chiamare `Suspend` (no QTE su reaction). Solo emit. Conserva la stack-depth=1 di §2.6.
- **Modifica retroattiva**: un subscriber **non può** cancellare/modificare effetti già committati nel batch. Può solo *aggiungere* nuovi effetti. Niente "replace damage 100 con 50".
- **Skip / consume**: niente "intercettazione" del damage prima che si applichi. Per quello esiste già `KernelEffect::PreApplyHook(name)` come pattern futuro (M018+) — non in scope ora.
- **Priorità custom**: niente "questa passiva risolve prima di quella". Si gestisce con `(owner_entity, registration_order)`.
- **`IdempotencyScope` custom (closure)**: in M017 solo le 4 varianti enum chiuse (`OncePerAction` / `OncePerCause` / `OncePerTarget` / `EveryTime`). Scope arbitrari (es. "una volta per nemico marked + alleato vivo") → si esprimono via `Custom` predicate dentro l'handler della reaction stessa, non nello scope. Valutare un `IdempotencyScope::Custom(name)` solo se M017 evidence mostra scope ricorrenti non coperti.

Se servono priorità esplicite o intercettazione, è un'estensione kernel-side futura, esplicita, con bump major.

## E — Mapping sull'esistente

| Pattern attuale | Cosa diventa in M017 |
|---|---|
| `FollowUpIntent` queue (M015) | **Subsume completa**: `FollowUpIntent` sparisce, diventa `KernelEffect::EnqueueAction { actor, action }` emesso via cascade. Niente queue dedicata. Cascade generica copre tutti i casi (singolo damage, status, signal, action). Riscrittura M015 isolata in **S03e** (~60 file di test adattati). |
| `CombatEvent` bus (esistente) | Resta single-source-of-truth. La cascade lo usa come notification channel, non crea un secondo bus. |
| `resolution.rs` (`apply_effects`) | Refactor: invece di processare `Vec<Effect>` da una skill, processa una queue di `KernelEffect` con riarmamento (push durante apply). Cuore di S03b. |
| `transitions_for_action_checked` (contract test M016) | Esteso: ogni blueprint subscriber dichiara quali `CombatEventKind` ascolta nel manifest, kernel verifica al boot che la handler esista. |

## F — Slice impact

- **S03b** include: `SkillBehavior` trait + registry + ctx + **drain loop** + `IdempotencyScope` registry + `CauseChain` propagation + contract test al boot.
- **S03e** isola la subsume di `FollowUpIntent` (riscrittura M015 dentro M017).

Vedi §5 per dipendenze.

## G — Riserva di revisione esplicita

Il modello `IdempotencyScope` a 4 varianti chiuse è **OK con riserva**: durante S03/S03b/S05 si rivedono i pattern reattivi delle skill che verranno definite per i 6 Rookie. Se emergono pattern non coperti dalle 4 varianti, si valuta `IdempotencyScope::Custom(name)` come 5ª variante. Decisione lazy: aggiungerla solo quando l'evidenza la richiede, non in anticipo.
