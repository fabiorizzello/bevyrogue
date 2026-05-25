# S06 Research: Windowed visual signoff remediation

## Summary

S06 is the **manual visual signoff** slice for M004. The milestone's Final Integrated
Acceptance has two halves: automated contract proof (delivered across S01–S05) and a
**human "looks good" signoff** for Sharp Claws, Baby Flame, and Baby Burner in `cargo winx`.
The user explicitly asked to be asked to review (K001).

The defining constraint: **auto-mode cannot run the windowed binary (K001).** So the
automatable deliverable of S06 is *the framework that lets the human sign off* — a UAT
runbook, a capture helper script, and a signoff/waiver artifact — plus the act of
actually soliciting the user's review. The human's pass/fail (or a formal waiver) is
recorded in the artifact; auto-mode never launches the window itself.

**This is a near-exact repeat of M002/S06**, which closed R014 the same way: "windowed
UAT runbook + capture script delivered … live data pending manual capture per K001
(auto-mode cannot launch windowed binary); framework complete." Use it as the template.

## Implementation Landscape

### Prior art to mirror (read these first)
- `scripts/capture-windowed-smoke.sh` — M002/S06 capture helper. Tees
  `cargo run --features windowed --bin bevyrogue` (stdout+stderr) into a timestamped log
  under `.gsd/milestones/M002/slices/S06/uat-evidence/`. Has an explicit K001 banner:
  "auto-mode must NOT invoke this script — only the human operator launches the windowed
  binary." S06 needs an M004-pointed equivalent (new evidence dir, and prefer the human
  alias `cargo winx`, see below).
- `.gsd/milestones/M002/slices/S06/S06-UAT.md` — the runbook/checkbox structure: a UAT
  checklist with a final unchecked "UAT evidence log (manual run per K001) —
  environment-limited" line. This is the honest "framework done, human step pending"
  marker shape.
- `.gsd/milestones/M002/slices/S06/uat-evidence/*.log` — example captured evidence logs.
- `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md` — the authoritative
  handoff. Its "What S06 still owns" section explicitly scopes S06: K001 manual signoff
  for all three skills, no strict-additive claim (D037 defers that). Read it; do not
  re-derive what S05 already delivered.
- `.gsd/milestones/M004/slices/S05/S05-UAT.md` — "Not Proven By This UAT" lists exactly
  what S06 must cover: "claw streak shape, bloom glow intensity, frame timing."

### Launch command — use the human alias
`.cargo/config.toml` defines: `winx = "run --features 'dev windowed'"`. The human
verification entry point is **`cargo winx`** (the M002 script predates/ignores the alias
and uses raw `--features windowed --bin bevyrogue`). The S06 runbook should instruct the
human to run `cargo winx`; the capture script can wrap the same. Note `[env] BEVY_HEADLESS = "1"`
is set globally to stop agents accidentally opening a window — the human run is the
intended exception.

### What the three skills should show (acceptance criteria for the human's eye)
From M004-CONTEXT + S05 artifacts:
- **Sharp Claws** — a claw/slash VFX that did not exist before this milestone; pale
  yellow-white overbright streak, ttl 6 ticks, scale pop, blooms under HDR. Triggered by
  `on_enter SpawnParticle("sharp_claws_slash")` on the `sharp_claws_strike` AnimGraph node.
- **Baby Flame** — charge ember-swirl → fast launch → impact shard-fan, data-driven path.
- **Baby Burner** — detonate flash, data-driven path.
- All three should read as **real glow** (HDR + bloom + overbright colors), visibly
  better than the flat-alpha-quad placeholder. D037 means **no strict additive blending**
  is expected — set that expectation in the runbook so the reviewer doesn't fail it for a
  deferred item.

## Natural Seams (for task decomposition)

1. **UAT runbook artifact** (`S06-UAT.md` and/or a `docs/uat/` runbook): per-skill,
   step-by-step — launch `cargo winx`, start an encounter, trigger each skill, observe,
   and a pass/fail line per skill with the acceptance bar spelled out. Pure doc; no code.
2. **Capture helper script** (`scripts/capture-windowed-*.sh` for M004/S06, evidence dir
   `.gsd/milestones/M004/slices/S06/uat-evidence/`): mirror the M002 script with the K001
   banner. Pure shell; testable only by shellcheck/dry inspection, NOT by running it.
3. **Signoff / waiver artifact**: a dedicated doc (or section) that records the user's
   verdict per skill — PASS with notes, or a **formal waiver** (the roadmap explicitly
   allows "captured or formally waived"). This is the thing that flips the milestone's
   remaining unchecked visual requirement.
4. **Soliciting the review**: at execution, the agent must actually ask the user to run
   `cargo winx` and report back (use `ask_user_questions` / a clear prompt). This is the
   one step that needs a real human in the loop — plan for it as an explicit pause, not a
   thing auto-mode can self-complete.

## First Proof / Highest Risk

The risk is **false closure** — auto-mode marking visual signoff "done" without a human
ever looking. MEM079/MEM078/MEM030 and the S05 acceptance doc all stress: keep automated
proof and manual UAT strictly separate, and state pending/manual non-claims explicitly.
The first thing to get right is the **honest status model**: the framework (runbook +
script + template) is auto-completable; the human verdict is either captured (user runs
`cargo winx` and signs off) or formally waived — and the artifact must say which, with no
overclaim. M002/S06 left the evidence-log line unchecked and called the framework
complete; that is the acceptable terminal state if the user is unavailable to review.

## Verification

- `test -s .gsd/milestones/M004/slices/S06/S06-UAT.md` (runbook present, non-empty).
- `test -x scripts/capture-windowed-<name>.sh` and `bash -n` / shellcheck the script
  (syntax only — never execute it from auto-mode, K001).
- Grep the script for the K001 "do not invoke from auto-mode" banner.
- Signoff/waiver artifact present and explicitly states per-skill PASS or formal waiver.
- Re-run S05's automated set (still green) to confirm S06 introduced no regression:
  `cargo test --test animation vfx_asset_load`, `vfx_asset_eval`,
  `render_no_vfx_kind_guard`; `cargo check --features windowed`;
  `cargo test --features windowed --test windowed_only vfx_rendering_acceptance`.
- **Do NOT** run `cargo winx` from auto-mode. The only valid proof of "looks good" is the
  user's manual sign-off captured in the artifact (or the formal waiver).

## Skills Discovered

No new skills installed. Relevant already-available skills:
- **bevy** — Bevy ECS / windowed render context if any render-side tweak is needed
  (unlikely; S05 already delivered the render path).
- **verify** / **verify-before-complete** — discipline for not claiming the visual bar is
  met without evidence in-message; directly applicable to the false-closure risk.
- **handoff** — relevant if the user is unavailable and S06 must terminate in the
  "framework complete, human capture pending" state for a later session.

## Requirements

`.gsd/REQUIREMENTS.md` has no Active R-records for M004; the R002/R004/R005 labels in
M004-CONTEXT are inherited/local constraints (MEM078). S06 owns no new headless
requirement — its job is the K001 manual-signoff boundary. Precedent: R014 was validated
in M002 with the manual capture left pending per K001 and the framework declared complete.
