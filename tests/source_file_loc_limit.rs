//! Architectural guard: no `src/**/*.rs` file may exceed the LOC cap.
//!
//! This is a structural assertion, not a feature test. It walks `src/`,
//! counts physical lines per `.rs` file, and fails listing every offender.
//! No allowlist by design — the suite is expected to fail until the oversized
//! files are split. Deterministic (only `std::fs`, sorted output).

use std::fs;
use std::path::{Path, PathBuf};

/// Hard cap on physical lines per source file.
const MAX_LOC: usize = 500;

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read directory {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("cannot read directory entry");
        let path = entry.path();
        let file_type = entry.file_type().expect("cannot stat directory entry");
        if file_type.is_dir() {
            collect_rs_files(&path, out);
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_source_file_exceeds_loc_cap() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src = Path::new(manifest_dir).join("src");

    let mut files = Vec::new();
    collect_rs_files(&src, &mut files);
    files.sort();

    let mut offenders: Vec<(PathBuf, usize)> = files
        .into_iter()
        .filter_map(|path| {
            let contents = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            let loc = contents.lines().count();
            (loc > MAX_LOC).then_some((path, loc))
        })
        .collect();

    // Largest first so the worst offenders are most visible.
    offenders.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    if !offenders.is_empty() {
        let mut report = format!(
            "{} source file(s) exceed the {MAX_LOC} LOC cap (split them into scoped submodules):\n",
            offenders.len()
        );
        for (path, loc) in &offenders {
            let rel = path
                .strip_prefix(manifest_dir)
                .unwrap_or(path.as_path())
                .display();
            report.push_str(&format!(
                "  {loc:>5} LOC  {rel}  (+{} over)\n",
                loc - MAX_LOC
            ));
        }
        panic!("{report}");
    }
}
