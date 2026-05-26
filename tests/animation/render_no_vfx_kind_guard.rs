//! M004/S02 T04 — headless grep-guard proving the legacy VFX-kind dispatch is gone.
//!
//! Success criterion 2: VfxParticleKind / kind_from_name / vfx_particle_kind no
//! longer exist in src/windowed/render.rs. This test pins it CI-provably in the
//! headless lane (no `windowed` feature) by embedding the source at compile time.
//!
//! Line comments are stripped before the check: historical comments still mention
//! the deleted type to explain what replaced it, and forbidding prose would be
//! noise. The guard targets the actual code construct, not the documentation.

const SRC: &str = include_str!("../../src/windowed/render.rs");
/// M006/S04 moved the authored-name->effect-id mapping out of render.rs into the
/// per-Digimon module; the data-driven boundary now spans both files.
const AGUMON_SRC: &str = include_str!("../../src/windowed/digimon/agumon/mod.rs");

const FORBIDDEN: [&str; 3] = ["VfxParticleKind", "vfx_particle_kind", "kind_from_name"];

/// Drop the `//...` line-comment tail from each line so only code is inspected.
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|line| line.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn render_rs_has_no_vfx_kind_dispatch() {
    let code = strip_line_comments(SRC);
    for ident in FORBIDDEN {
        assert!(
            !code.contains(ident),
            "src/windowed/render.rs still contains forbidden VFX-kind identifier `{ident}` in code (T03 was supposed to delete the enum + string-match dispatch)"
        );
    }
}

/// Positive contract: the data-driven spawn boundary that REPLACED VFX-kind
/// dispatch must still be present. M006/S04 generalized it into a registry: the
/// engine reads `OnEnterEffectRegistry` (no closed match), and the per-Digimon
/// module maps each authored SpawnParticle name to owned effect id(s). If this
/// name->effect-id seam ever disappears, a future agent can localize that the data
/// path regressed before the forbidden-identifier guard above would even fire.
#[test]
fn render_rs_keeps_the_data_driven_effect_id_boundary() {
    let code = strip_line_comments(SRC);
    assert!(
        code.contains("OnEnterEffectRegistry"),
        "src/windowed/render.rs must read the OnEnterEffectRegistry name->effect-id boundary that replaced VFX-kind dispatch"
    );
    // Sharp Claws must be wired through the owned effect id, not a hardcoded kind.
    let agumon = strip_line_comments(AGUMON_SRC);
    assert!(
        agumon.contains("sharp_claws.slash"),
        "Sharp Claws must route through the owned effect id, not a hardcoded VFX-kind path"
    );
}
