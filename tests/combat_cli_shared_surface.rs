use std::{fs, path::Path, process::Command};

struct CliRun {
    success: bool,
    status: String,
    output: String,
}

fn run_combat_cli_proof() -> CliRun {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target_dir = manifest_dir.join("target");
    fs::create_dir_all(&target_dir).expect("target directory should be creatable for cwd proof");

    let output = Command::new(env!("CARGO_BIN_EXE_combat_cli"))
        .current_dir(&target_dir)
        .env("BEVYROGUE_JSONL", "1")
        .env("BEVYROGUE_CLI_PROOF", "1")
        .env("BEVYROGUE_CLI_TICK_LIMIT", "120")
        .output()
        .expect("combat_cli binary should launch via CARGO_BIN_EXE_combat_cli");

    let mut combined = String::new();
    combined.push_str("--- stdout ---\n");
    combined.push_str(&String::from_utf8_lossy(&output.stdout));
    combined.push_str("\n--- stderr ---\n");
    combined.push_str(&String::from_utf8_lossy(&output.stderr));

    CliRun {
        success: output.status.success(),
        status: output.status.to_string(),
        output: combined,
    }
}

fn assert_contains(output: &str, needle: &str) {
    assert!(
        output.contains(needle),
        "expected combat_cli proof output to contain `{needle}`\n{output}"
    );
}

fn assert_not_contains(output: &str, needle: &str) {
    assert!(
        !output.contains(needle),
        "combat_cli proof output unexpectedly contained `{needle}`\n{output}"
    );
}

#[test]
fn combat_cli_binary_emits_shared_combat_surfaces_from_non_root_cwd() {
    let run = run_combat_cli_proof();

    assert!(
        run.success,
        "combat_cli proof process should exit successfully, status={}\n{}",
        run.status, run.output
    );

    assert_contains(&run.output, "Action affordances");
    assert_contains(&run.output, "OnCombatBeat");
    assert_contains(&run.output, "OnKernelTransition");
    assert_contains(&run.output, "OnActionResolved");
    assert_contains(&run.output, "OnDamageDealt");
    assert_contains(&run.output, "OnSkillCast");
    assert_contains(&run.output, "[CLI_PROOF] validation_snapshot:");
    assert_contains(&run.output, "support=grace=");

    // Hidden startup drift can still appear in output even when a worker-thread
    // panic or missing-message path exits 0, so forbid those markers explicitly.
    assert_not_contains(&run.output, "panicked");
    assert_not_contains(&run.output, "Message not initialized");
    assert_not_contains(&run.output, "[QUERY] Skill book unavailable");
    assert_not_contains(&run.output, "validation_snapshot_error");
    assert_not_contains(&run.output, "readiness_timeout");
    assert_not_contains(&run.output, "[CLI_PROOF] failure");
}

#[test]
fn combat_cli_source_stays_on_shared_surfaces_not_presentation_metadata() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = manifest_dir.join("src/bin/combat_cli.rs");
    let source = fs::read_to_string(&source_path).expect("combat_cli source should be readable");

    for shared_surface in [
        "build_snapshot_from_ecs_with_sp",
        "query_action_affordance",
        "first_enabled_target_id",
        "capture_validation_snapshot",
    ] {
        assert!(
            source.contains(shared_surface),
            "combat_cli should name shared surface `{shared_surface}` in {}",
            source_path.display()
        );
    }

    assert!(
        !source.contains("animation_sequence"),
        "combat_cli must not branch on presentation animation metadata"
    );
    assert!(
        !source.contains("qte"),
        "combat_cli must not branch on presentation QTE metadata"
    );
    assert!(
        !source.contains(".holy_support"),
        "combat_cli must not depend on retired holy_support snapshot fields"
    );
    assert!(
        !source.contains(".twin_core"),
        "combat_cli must not depend on retired twin_core snapshot fields"
    );
}
