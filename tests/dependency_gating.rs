//! Dependency-graph gating guard (M005/S04/T01).
//!
//! The central S04 risk is dependency leakage: `bevy_enoki` 0.6 hard-depends on
//! the entire Bevy render stack (`bevy_render`, `bevy_sprite_render`,
//! `bevy_core_pipeline`, `bevy_camera`, `bevy_mesh`, `bevy_shader`). If it were
//! reachable from the default/headless build it would violate the headless
//! dep-isolation rules (R002/R005/R016) and balloon the headless build.
//!
//! This is a standalone single-binary test domain (R003): it is NOT under
//! `tests/windowed_only/` and is NOT `#![cfg(feature = "windowed")]`, so it runs
//! on the default `dev` headless build. It shells out to `cargo tree --invert`
//! to inspect the actual resolved dependency graph for each feature set:
//!   - headless (`--features dev`): `bevy_enoki` MUST be absent.
//!   - windowed (`--features windowed`): `bevy_enoki` MUST be present.
//!
//! On failure the captured `cargo tree` output is logged so a future agent can
//! read the graph without re-running anything.

use std::process::Command;

/// Run `cargo tree -e normal --no-default-features --features <features>
/// --invert bevy_enoki --offline` from the package root and capture the result.
fn cargo_tree_invert(features: &str) -> std::process::Output {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "tree",
            "-e",
            "normal",
            "--no-default-features",
            "--features",
            features,
            "--invert",
            "bevy_enoki",
            "--offline",
        ])
        .output()
        .expect("failed to invoke `cargo tree`")
}

#[test]
fn bevy_enoki_absent_from_headless_graph() {
    let out = cargo_tree_invert("dev");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // `cargo tree --invert <pkg>` exits non-zero when the package is not in the
    // resolved graph (no package matches the spec). We assert on the failure
    // signal rather than cargo's exact error phrasing, which is not stable.
    assert!(
        !out.status.success(),
        "expected `cargo tree --invert bevy_enoki` to FAIL for the headless \
         (`dev`) feature set, proving bevy_enoki is absent from the headless \
         graph (R005). It succeeded instead.\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
    // Defensive: even if cargo's exit semantics ever change, the inverted tree
    // must not contain bevy_enoki under the headless build.
    assert!(
        !stdout.contains("bevy_enoki"),
        "bevy_enoki leaked into the headless (`dev`) dependency graph (R005).\n\
         --- inverted tree (stdout) ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
}

#[test]
fn bevy_enoki_present_in_windowed_graph() {
    let out = cargo_tree_invert("windowed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        out.status.success(),
        "expected `cargo tree --invert bevy_enoki` to SUCCEED for the \
         `windowed` feature set, proving bevy_enoki is present.\n\
         --- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
    assert!(
        stdout.contains("bevy_enoki"),
        "bevy_enoki missing from the `windowed` dependency graph; the windowed \
         feature must pull `dep:bevy_enoki`.\n--- inverted tree (stdout) ---\n{stdout}"
    );
}
