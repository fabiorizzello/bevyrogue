/// Integration-level demo gate for slice S02 / demo gate 2:
/// `validate_timeline_refs` reports a dangling-reference typo with axis + site.
use bevyrogue::combat::api::{
    registry::ExtRegistries,
    timeline::{Beat, BeatEdge, BeatKind, CompiledTimeline, validate_timeline_refs},
};

#[test]
fn validate_timeline_refs_catches_typo_in_hook_id() {
    let timeline = CompiledTimeline {
        id: "typo_skill",
        entry: "cast",
        beats: vec![Beat {
            id: "cast",
            kind: BeatKind::Cast,
            // Typo: "on_turn_staart" instead of "on_turn_start"
            hook: Some("on_turn_staart"),
            selector: None,
            presentation: None,
            payload: None,
        }],
        edges: vec![BeatEdge {
            from: "cast",
            to: "impact",
            gate: None,
        }],
    };

    let regs = ExtRegistries::default(); // nothing registered

    let result = validate_timeline_refs(&timeline, &regs);
    assert!(result.is_err(), "expected Err for unregistered hook id");

    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].axis, "hook");
    assert_eq!(errors[0].missing_id, "on_turn_staart");
    assert_eq!(errors[0].site, "beat cast");
}
