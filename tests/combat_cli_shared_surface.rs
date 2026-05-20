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

// Spawna il binario combat_cli come sottoprocesso (~0.3s di startup): è
// l'unico test non trascurabile della suite. Escluso dall'esecuzione
// standard; runnalo esplicitamente con `cargo test -- --ignored`.
#[test]
#[ignore = "spawns combat_cli subprocess (~0.3s); run with --ignored"]
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
    assert_contains(&run.output, "grace=");

    // Hidden startup drift can still appear in output even when a worker-thread
    // panic or missing-message path exits 0, so forbid those markers explicitly.
    assert_not_contains(&run.output, "panicked");
    assert_not_contains(&run.output, "Message not initialized");
    assert_not_contains(&run.output, "[QUERY] Skill book unavailable");
    assert_not_contains(&run.output, "validation_snapshot_error");
    assert_not_contains(&run.output, "readiness_timeout");
    assert_not_contains(&run.output, "[CLI_PROOF] failure");
}
