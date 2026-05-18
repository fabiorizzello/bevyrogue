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

/// Concatenates `src/bin/combat_cli.rs` plus every `.rs` under the
/// `src/bin/combat_cli/` directory module. The binary was split into a
/// directory module; the guard must cover the whole tree so splitting a file
/// can never hide a shared surface or smuggle in presentation metadata.
fn combat_cli_module_source(manifest_dir: &Path) -> String {
    fn collect(path: &Path, out: &mut String) {
        if path.is_dir() {
            let mut entries: Vec<_> = fs::read_dir(path)
                .unwrap_or_else(|e| panic!("cannot read dir {}: {e}", path.display()))
                .map(|e| e.expect("dir entry").path())
                .collect();
            entries.sort();
            for entry in entries {
                collect(&entry, out);
            }
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push_str(
                &fs::read_to_string(path)
                    .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display())),
            );
            out.push('\n');
        }
    }

    let mut source = String::new();
    collect(&manifest_dir.join("src/bin/combat_cli.rs"), &mut source);
    let dir = manifest_dir.join("src/bin/combat_cli");
    if dir.is_dir() {
        collect(&dir, &mut source);
    }
    assert!(!source.is_empty(), "combat_cli source should be readable");
    source
}

#[test]
fn combat_cli_source_stays_on_shared_surfaces_not_presentation_metadata() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = combat_cli_module_source(manifest_dir);

    for shared_surface in [
        "build_snapshot_from_ecs_with_sp",
        "query_action_affordance",
        "first_enabled_target_id",
        "capture_validation_snapshot",
    ] {
        assert!(
            source.contains(shared_surface),
            "combat_cli should name shared surface `{shared_surface}` somewhere in its module tree"
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
