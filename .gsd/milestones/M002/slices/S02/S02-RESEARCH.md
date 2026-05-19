# Slice S02 Research: Basic Attack + Two-Clock Impact Barrier + Telegraph Chip

**Milestone**: M002  
**Slice**: S02  
**Risk**: high  
**Depends**: S01  

## Summary

S02 delivers the kernel-animation handshake for frame-accurate impact-synchronized damage. The core blocker is in `src/combat/turn_system/pipeline/timeline_exec.rs` line 349-351: `BeatRunner::run_to_completion()` auto-resumes `AwaitingCue` instead of suspending, breaking the two-clock invariant. S02 must make the kernel properly suspend on `ReleaseKernelCue`, restore that control to the player layer, and wire the handshake so Sharp Claws windup→strike→recovery plays through before damage lands (framed by kernel Intent emission).

The slice contains five natural seams:
1. **Kernel suspend logic** (timeline_exec): block auto-resume, let kernel halt on `AwaitingCue`
2. **Player KernelCue predicate** (player.rs): evaluate `Predicate::KernelCue`, drive resume flow
3. **Sharp Claws animation graph** (anim_graph.ron): windup/strike/recovery nodes + ReleaseKernelCue cue at impact frame
4. **Anti-DRY mul:18 remediation** (skills.ron + validation): unify duplicate damage multipliers behind a shared param
5. **I3 cue-handshake test** (integration test): verify HeadlessAuto ≡ Windowed Intent stream with kernel cue in middle

## Active Requirements/Constraints

**R002 (Headless-first)**: Player KernelCue logic must be testable headless. The handshake between animation frame arrival (`ReleaseKernelCue` cue) and kernel Intent enqueue must not depend on windowed rendering or input.

**R004 (Determinism)**: No wall-clock or unseeded RNG. The two-clock barrier timing is deterministic: same anim frames, same kernel ordering, always.

**R005 (No windowed deps outside windowed)**: `KernelCue` predicate evaluation stays in the kernel-agnostic `AnimGraphPlayer` FSM. Windowed-specific logic (`Clock::Windowed` stall/resume) lives only in tests or feature-gated UI.

**R006 (Repo hygiene)**: S01 validation test `anim_gameplay_command_forbidden.rs` must pass: agumon anim_graph.ron has zero gameplay commands (D001). S02 extends validation to reject gameplay commands in cue Presentations.

**I3 (Intent parity)**: `tests/timeline_two_clock_parity.rs` must remain green. HeadlessAuto and Windowed runs produce identical Intent stream debug-format strings (only timing differs).

**D001 (Gameplay×Anim boundary)**: Kernel payload (damage, status, buffs) lives in `skills.ron` beats only. Animation graph cues carry **only** presentation commands (particles, shake, VFX) or `ReleaseKernelCue` signals — never gameplay.

## Implementation Landscape

### Files in Scope

- **`src/combat/turn_system/pipeline/timeline_exec.rs`** (263 lines)  
  Line 117-131: Builds `BeatRunner` and calls `run_to_completion()`.  
  **Gap**: `run_to_completion()` auto-resumes `AwaitingCue` (runner.rs:349-351), masking the cue handshake.  
  **Fix**: Prevent auto-resume; let execution halt and propagate `AwaitingCue` or equivalent to caller.

- **`src/combat/runtime/runner.rs`** (386 lines)  
  Lines 322-329: `resume_cue()` unlatch method exists.  
  Lines 337-358: `run_to_completion()` auto-resumes on `AwaitingCue`.  
  Lines 293-296 (linear path) and 202-204 (loop body): stall logic: `if beat.presentation.is_some() && self.clock == Clock::Windowed → return AwaitingCue`.  
  **Intent**: These are correct; the issue is the `run_to_completion()` wrapper consuming them.

- **`src/animation/player.rs`** (106 lines)  
  Lines 23-41: `advance()` FSM. Currently ignores `Predicate::KernelCue` (line 39 filters to `Always` and `TimeInNode` only).  
  **Gap**: Must evaluate `KernelCue` transitions; needs `resume_cue()` signal from caller.  
  **Seam**: Add `KernelCue` to the predicate filter and expose a method to accept the resume signal.

- **`src/animation/anim_graph.rs`** (385 lines)  
  Lines 81-96: `FrameCue` and `FrameCueCommand` schema (S01 delivered).  
  Lines 217-218: `Predicate::KernelCue` variant (S01 delivered).  
  **S02 use**: Author Sharp Claws graph with cues carrying `ReleaseKernelCue` command.

- **`assets/digimon/agumon/anim_graph.ron`** (43 lines)  
  Currently has only `baby_flame_*` nodes (Baby Flame skill).  
  **S02 author**: Add Sharp Claws skill: `sharp_claws_windup` (0-10 frames, on_enter: particle or stance cue), `sharp_claws_strike` (11-15 frames, cues: ReleaseKernelCue at frame 14), `sharp_claws_recovery` (16-25 frames).

- **`assets/data/digimon/agumon/skills.ron`** (186 lines)  
  Lines 1-45: Baby Flame skill (damage: 18, uses timeline).  
  Lines 64-93: Agumon Follow-Up (damage: 22).  
  Lines 96-109: Greymon Basic (damage: 16).  
  **DRY issue**: Line 15 `mul: 18` and line 83 `amount: 22` are inlined. S02 must centralize to `Static` param refs in the timeline `DealDamage` beats (e.g. `mul: Static(ParamKey("sharp_claws_damage_mult"))`), then validate all are defined.

- **`tests/timeline_two_clock_parity.rs`** (203 lines)  
  I3 test: HeadlessAuto ≡ Windowed Intent stream (only timing differs).  
  **S02 scope**: Extend to include a `ReleaseKernelCue` cue between beats; assert both clocks emit identical Intent stream before and after the cue fires.

- **`tests/anim_gameplay_command_forbidden.rs`** (122 lines)  
  Anti-DRY gate: Production agumon graph must have zero gameplay commands (D001).  
  **S02 scope**: Test passes as-is; validation already rejects `EmitDamage` in cue Presentations (lines 61-87).

- **`src/animation/validation/graph.rs`** (200+ lines)  
  Calls `validate_command()` on `on_enter` and cue Presentations.  
  **S02 scope**: Already rejects gameplay in cues; no changes needed.

### Relationships

```
timeline_exec.rs (kernel executor)
    ↓ calls
runner.rs (BeatRunner FSM)
    ↓ emits StepOutcome
turn_pipeline.rs (handles timeline_exec_outcome)

AnimGraphPlayer (anim FSM) ← MUST INTEGRATE WITH KERNEL HANDSHAKE
    ↓ reads
anim_graph.ron (skill sequencing, cues, predicates)
    ↓ checked by
validation/graph.rs (gameplay-command boundary)

skills.ron (kernel payload, timeline def)
    ↓ compiles to
CompiledTimeline (beats, hooks, intents)
```

## Natural Seams

### 1. Kernel Suspend-on-AwaitingCue (timeline_exec + runner)

**File**: `src/combat/turn_system/pipeline/timeline_exec.rs:117-131` + `src/combat/runtime/runner.rs:337-358`

**Current behavior**:
```rust
let outcome = unsafe {
    runner.run_to_completion(world, &*regs_ptr, SkillCtxMode::Execute, &mut pending, 1024)
};
// Auto-resumes AwaitingCue internally (runner.rs:349-351):
// StepOutcome::AwaitingCue => { self.resume_cue(); }
```

**S02 change**: Make kernel halt when it hits a `ReleaseKernelCue`-triggered presentation beat.  
**Approach**: Do NOT auto-resume in `run_to_completion()`. Instead:
- Let `AwaitingCue` propagate to `timeline_exec` caller.
- `timeline_exec` signals to the turn pipeline that action is suspended.
- Turn pipeline yields to the animation-player frame update loop.
- Animation player eventually sees the `ReleaseKernelCue` cue, evaluates `Predicate::KernelCue` transition.
- Animation player calls `resume_cue()` on a shared runner reference.
- Turn pipeline resumes `run_to_completion()` from the kernel's stalled state.

**Risk**: Requires threading a runner reference or message channel through the turn pipeline. Must not break determinism.

### 2. Player KernelCue Predicate + Resume Handshake (player.rs)

**File**: `src/animation/player.rs:25-41`

**Current behavior**: Ignores `Predicate::KernelCue` (line 39 filters only `Always` and `TimeInNode`).

**S02 change**: 
- Evaluate `KernelCue` predicates in the `advance()` loop alongside `TimeInNode`.
- Add a method `fire_kernel_cue()` or similar that the caller invokes when `ReleaseKernelCue` is emitted from the timeline.
- This method signals the next transition to take, or sets an internal flag that unblocks a stalled `KernelCue` gate.

**Seam**: Split responsibility:
- `advance()` returns sprite frame; caller checks for `ReleaseKernelCue` cues in the anim_graph at current frame.
- If cue present, caller invokes `fire_kernel_cue()` to unlock `KernelCue` transitions.
- Next `advance()` sees the fired cue and evaluates the gate.

**Test target**: `tests/anim_player_fsm.rs` — add a test for `KernelCue` transition gating.

### 3. Sharp Claws Animation Graph Authoring (anim_graph.ron)

**File**: `assets/digimon/agumon/anim_graph.ron:43 lines`

**Current**: Only `baby_flame_*` skill nodes.

**S02 author**: Add Sharp Claws skill (Agumon's basic attack).
```ron
"sharp_claws_windup": (
    frames: (0, 10),
    on_enter: [
        SpawnParticle(
            name: "sharp_claws_spin",
            origin: CasterCenter,
            motion: Static,
        ),
    ],
),
"sharp_claws_strike": (
    frames: (11, 15),
    cues: [
        (at: 14, command: ReleaseKernel(())),
    ],
),
"sharp_claws_recovery": (
    frames: (16, 25),
),
```

**Clip ranges**: Extends `clip.ron` with new range `"sharp_claws": (start: X, end: Y)` pointing to sprite sheet slices.

**Transitions**:
```ron
(from: "sharp_claws_windup", to: Node("sharp_claws_strike"), when: TimeInNode),
(from: "sharp_claws_strike", to: Node("sharp_claws_recovery"), when: TimeInNode),
(from: "sharp_claws_recovery", to: Exit, when: TimeInNode),
```

**Validation**: No gameplay commands in `on_enter` or cues (D001 check passes).

### 4. Anti-DRY mul:18 Remediation (skills.ron + validation)

**File**: `assets/data/digimon/agumon/skills.ron:1-45, 64-93, etc.`

**Current issue**: Damage multipliers inlined in each skill's timeline.  
Line 15: `mul: 18` (Baby Flame)  
Line 83: `amount: 22` (Agumon Follow-Up)  
Line 108: `amount: 16` (Greymon Basic)

**S02 remediation**:
1. Define a params table in the skill (or globally in SkillBook):
   ```ron
   params: {
       "sharp_claws_damage_mult": 18,
       "agumon_follow_up_damage_mult": 22,
       "greymon_basic_damage_mult": 16,
   }
   ```
2. Change timeline beats to use `Static` param refs:
   ```ron
   (id: "impact_damage", kind: Impact, hook: Some("core/deal_damage"), 
    selector: Some("core/primary"), 
    presentation: None, 
    payload: Some(DealDamage(amount: 18, tag: Fire, target: Single))),
   ```
   becomes (in the kernel's data layer, not anim_graph):
   ```ron
   payload: Some(DealDamage(amount: Static(ParamKey("sharp_claws_damage_mult")), tag: Fire, target: Single))
   ```

3. **Validation**: Extend `src/animation/validation/command.rs:validate_param_ref()` to warn if a `Static` param is not found in the skill's param table.

**Note**: This is a kernel-side change, not animation-side. S02 separates the concern: animation graph stays pure presentation, kernel params live in skills.ron.

### 5. I3 Cue-Handshake Test (integration test)

**File**: `tests/timeline_two_clock_parity.rs:1-203`

**Current test**: Two-beat timeline (Cast with Presentation → Impact). Asserts HeadlessAuto ≡ Windowed Intent stream.

**S02 extend**:
- Add a third beat: Cast → Impact (with ReleaseKernelCue cue at frame 1) → Recovery.
- Verify both clocks produce identical Intent stream in the same order.
- Assert that `AwaitingCue` stall occurs at the cue-carrying beat in Windowed mode.
- Assert that HeadlessAuto does not stall (auto-resumes — S02 will change this; update accordingly).

**Assertion**: `I3 extended` — Intent parity holds across the cue handshake.

## First Proof: Kernel Suspend-on-AwaitingCue Wiring

**Highest risk / biggest unblocker**:  
The two-clock barrier is **not yet wired**. `run_to_completion()` auto-resumes `AwaitingCue` instead of allowing the kernel to suspend and wait for the animation player to signal `resume_cue()`.

**First proof task**: Make `run_to_completion()` NOT auto-resume; instead propagate `AwaitingCue` or a "cue-awaiting" signal. Verify:
1. `cargo test timeline_two_clock_parity --lib` fails or changes behavior (expected; the auto-resume masking goes away).
2. Update the test to expect the new behavior: kernel halts, test manually calls `resume_cue()`, then continues.
3. Verify `cargo build --features windowed` still compiles (no new deps introduced).
4. Verify `cargo test --lib` passes (no other tests broken by the suspend change).

**Concrete command**:
```bash
# Before: run_to_completion auto-resumes; stall is invisible.
cargo test timeline_two_clock_parity -- --nocapture

# Expected (S02 implementation): test will need to adapt to kernel suspending.
```

## Verification

**Unit Tests** (in `src/` tree):
- `cargo test --lib` — all existing tests remain green.
- Add `tests/anim_player_fsm.rs::player_kernel_cue_transition_blocks_until_fired` — verify `KernelCue` gate blocks and fires on signal.

**Integration Tests** (in `tests/` tree):
- `cargo test timeline_two_clock_parity` — extend to include cue; assert Intent parity holds.
- `cargo test anim_gameplay_command_forbidden` — anti-DRY gate remains green (S02 doesn't author gameplay commands in anim_graph).

**Feature builds**:
- `cargo build --features windowed` — no errors (R005).
- `cargo build --no-default-features` — headless mode builds cleanly.

**Invariants**:
- **I3** (Intent parity): HeadlessAuto and Windowed Intent streams are debug-format identical (timing only differs).
- **R002** (Headless-first): `KernelCue` predicate evaluation is testable without windowed/wgpu.
- **D001** (Gameplay×Anim): `anim_gameplay_command_forbidden.rs::agumon_graph_has_no_gameplay_commands` passes.
- **R003** (Clip↔Atlas parity): New Sharp Claws frames must map correctly to clip.ron ranges.

## Risks/Unknowns

1. **Threading runner through turn pipeline**: Kernel suspend requires a way for the turn pipeline to know that `run_to_completion()` halted at `AwaitingCue`. May need a wrapper state or message channel. Determinism must be preserved (no callbacks, pure state).

2. **Animation player API surface**: Adding `fire_kernel_cue()` or similar requires deciding when the caller invokes it. Is it called by the turn pipeline (tight coupling) or the rendering loop (looser)? S02 should clarify ownership.

3. **Clip atlas parity**: New Sharp Claws sprite frames must fit in the existing clip.ron ranges. If sprite sheet changes, R003 test (`tests/anim_graph_asset.rs`) might fail.

4. **Mul:18 DRY remediation scope**: S02 focuses on Sharp Claws only. Refactoring all existing skills (Baby Flame, Greymon, etc.) to use param refs is out of scope but flagged for S03.

5. **HeadlessAuto behavior change**: S02 changes `run_to_completion()` to NOT auto-resume. Existing headless tests that relied on auto-resume will need updates.

## Skills Discovered

- **bevy**: ECS world mutations, message-based event system, Arc safety for shared resources.
- **rust-best-practices**: FSM design patterns (state machines with clear state transitions), lifetimes for borrowed data, feature-gated code.
- **rust-testing**: Property-based testing (stream parity), deterministic fixtures, `cargo test --features X` targeting.

## Sources

- **M001 context**: `S01` slice delivered schema, predicates, registry, player core.
- **Kernel architecture**: `src/combat/runtime/runner.rs` (BeatRunner FSM, stall gates, resume protocol).
- **Animation schema**: `src/animation/anim_graph.rs` (FrameCue, ReleaseKernelCue, Predicate::KernelCue).
- **Validation tests**: `tests/anim_gameplay_command_forbidden.rs` (D001 gate), `tests/timeline_two_clock_parity.rs` (I3 parity).
- **Asset spec**: `assets/digimon/agumon/` (anim_graph.ron, clip.ron, skills.ron).
- **Integration points**: `src/combat/turn_system/pipeline/timeline_exec.rs` (turn pipeline entry), `src/animation/player.rs` (FSM advance loop).
