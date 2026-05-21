//! Unit-level contract for `validate_timeline_refs` on `BeatKind::Loop` —
//! the integration tests in `tests/compiled_timeline_builtin_validation.rs`
//! and `tests/timeline_validate_typo.rs` exercise `hook` and `predicate` axes
//! on `Cast`/`Impact` beats + edges, but NEVER exercise a Loop beat with a
//! missing `exit_when` predicate. The site format for a Loop is
//! `"beat <loop_id>"` (not `"edge from->to"`), which is the unique contract
//! asserted here.
use bevyrogue::combat::runtime::{
    registry::ExtRegistries,
    timeline::{Beat, BeatEdge, BeatKind, CompiledTimeline, validate_timeline_refs},
};

#[test]
fn missing_loop_exit_when_predicate_reported_with_loop_beat_site() {
    let tl = CompiledTimeline {
        id: "bad_loop",
        entry: "loop_beat",
        beats: vec![Beat {
            id: "loop_beat",
            kind: BeatKind::Loop {
                body: vec![],
                exit_when: "missing_exit_pred",
            },
            hook: None,
            selector: None,
            presentation: None,
            payload: None,
        }],
        edges: Vec::<BeatEdge>::new(),
    };
    let regs = ExtRegistries::default();
    let err = validate_timeline_refs(&tl, &regs).unwrap_err();
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].axis, "predicate");
    assert_eq!(err[0].missing_id, "missing_exit_pred");
    assert_eq!(err[0].site, "beat loop_beat");
}
