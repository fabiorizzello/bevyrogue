# S05: Full Agumon VFX migration to enoki — UAT

**Milestone:** M005
**Written:** 2026-05-26T09:50:01.101Z

# S05 UAT: Full Agumon VFX migration to enoki

## UAT Type

**Split**: Integration (structural/contract surface, all four build variants) verified headless by auto-mode. Visual quality (K001) is a **human/UAT gate** — auto-mode cannot launch the windowed binary; the user must run `cargo winx` to sign off.

## Preconditions

- Clean build of both flavors (`cargo build` + `cargo build --features windowed`)
- `cargo winx` alias available (or `cargo run --features windowed`)
- All four headless checks already green (see Verification section)

## Steps — Automated (already passed)

1. `cargo test` → 51 passed, 0 failed
2. `cargo build --features windowed` → exit 0
3. `cargo test --features windowed --test windowed_only` → 49 passed, 0 failed
4. `cargo test --test dependency_gating` → 2 passed

## Steps — Manual (K001, requires user)

5. Run `cargo winx`
6. Start a combat encounter; queue **Sharp Claws** on either combatant and observe the impact frame.
   - **Expected:** a quick pale yellow-white particle burst appears at the struck target's position; it fades within ~0.25 s; no flat quad quad visible.
7. Queue **Baby Flame** (projectile) and observe the impact when the projectile reaches the target.
   - **Expected:** an orange-white particle burst appears; it reads as a central flash plus radiating shards (larger than the pre-S05 flat quad); it fades quickly; no flat quad visible.
8. Queue **Baby Burner** and observe the detonate frame when the final hit lands.
   - **Expected:** a wider orange radial shard burst (larger spawn_amount / speed than impact) appears, folding the central flash pop; it fades in ~0.3 s; no flat quad visible.
9. Confirm all three bursts **look better than the flat-quad placeholder**.

## Expected Outcomes

- Steps 5–8: each skill's contact burst renders as a bevy_enoki one-shot particle effect, not a flat colored quad.
- Step 9: user signs off that the three enoki bursts are a visible improvement over the previous placeholder (K001).
- Dep-isolation: headless `cargo test` remains unaffected (bevy_enoki not linked).

## Edge Cases

- If a burst is absent (no particles), run `cargo winx` with `RUST_LOG=target=warn,windowed=warn` — the `diagnose_agumon_enoki_vfx_load` system will emit a WARN naming the failed asset by effect id so the load failure can be diagnosed by name.
- The quad path remains behind the seam as a reversible fallback; no quad should appear for the three routed ids.

## Not Proven By This UAT

- Visual correctness of the particle parameters (color, speed, lifetime, scale) beyond "better than placeholder" — these are tunable by editing the .particle.ron assets without code changes.
- Charge buildup and traveling projectile body VFX — these are deferred to a follow-up (D040); they still render via the quad path.
- Deletion of the now-dormant quad path for Agumon ids — deferred follow-up (D041).
