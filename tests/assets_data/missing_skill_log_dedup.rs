//! T03 (S07) — `missing_skill_log_once` deduplication + context.
//!
//! Post-S07 the combat panel consults the canonical `SkillBookHandle`, so a
//! `MissingSkill` is a *genuine* miss rather than the old arbitrary-partial-book
//! defect. To keep a true miss debuggable, the diagnostic must name the skill id
//! and the book handle consulted — and, because the panel runs every frame, must
//! deduplicate so each `(skill id, handle)` pair logs at most once.

use std::collections::HashSet;

use bevyrogue::data::missing_skill_log_once;

/// First sighting of a `(skill, handle)` pair yields a message naming both the
/// skill id and (in the loaded-handle case) signalling the book consulted;
/// the second sighting is suppressed.
#[test]
fn logs_once_then_deduplicates() {
    let mut seen: HashSet<String> = HashSet::new();

    let first = missing_skill_log_once(&mut seen, "diamond_storm", None);
    let message = first.expect("first sighting must produce a diagnostic");
    assert!(
        message.contains("diamond_storm"),
        "diagnostic must name the missing skill id; got: {message}"
    );
    assert!(
        message.contains("handle"),
        "diagnostic must name the book handle consulted; got: {message}"
    );

    let second = missing_skill_log_once(&mut seen, "diamond_storm", None);
    assert!(
        second.is_none(),
        "repeat sighting of the same (skill, handle) pair must be suppressed; got: {second:?}"
    );
}

/// Distinct skill ids are independently logged — dedup keys on the pair, not just
/// on having logged anything.
#[test]
fn distinct_skill_ids_each_log_once() {
    let mut seen: HashSet<String> = HashSet::new();

    assert!(missing_skill_log_once(&mut seen, "diamond_storm", None).is_some());
    assert!(missing_skill_log_once(&mut seen, "power_paw", None).is_some());
    assert!(missing_skill_log_once(&mut seen, "diamond_storm", None).is_none());
    assert!(missing_skill_log_once(&mut seen, "power_paw", None).is_none());
}

/// The absent-handle case names the fallback book explicitly, distinguishing a
/// miss during asset loading from a miss against a loaded canonical book.
#[test]
fn absent_handle_names_the_fallback_book() {
    let mut seen: HashSet<String> = HashSet::new();
    let message = missing_skill_log_once(&mut seen, "diamond_storm", None)
        .expect("first sighting must produce a diagnostic");
    assert!(
        message.contains("fallback"),
        "absent handle must be reported as the fallback book; got: {message}"
    );
}
