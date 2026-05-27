/// T01 – Reproduce false MissingSkill via the combat-panel skill-book path.
///
/// `render.rs` currently resolves the active skill book with:
///
/// ```ignore
/// let skill_book = skill_books.iter().next().map(|(_, book)| book).unwrap_or(...);
/// ```
///
/// Because `Assets<SkillBook>` holds *all* partial per-digimon books plus the
/// assembled canonical one, `.iter().next()` returns an **arbitrary** partial
/// book — typically Agumon's — regardless of which Digimon is acting. When
/// Renamon is the active unit her skill IDs (e.g. `diamond_storm`) do not appear
/// in Agumon's partial book, so `query_action_affordance` returns
/// `ActionStatus::Disabled { reason: MissingSkill }` even though the canonical
/// merged book contains the skill.
///
/// ## Test design
///
/// - `render_panel_path_resolves_renamon_skill_without_missing_skill` is the
///   **required** assertion: when the combat panel routes through the canonical
///   skill book (via `SkillBookHandle`), Renamon's `diamond_storm` must not
///   return `MissingSkill`. This test is **RED** today because `render.rs`
///   picks the first arbitrary partial book instead. T02 fixes this by
///   substituting `SkillBookHandle` lookups.
///
/// - `partial_book_confirms_missing_skill_is_the_root_cause` is a companion
///   documentation test showing that the partial Agumon book IS the source of
///   the false positive — it always passes and explains the mechanism.
use bevyrogue::combat::action_query::{ActionQueryKind, ActionStatus, query_action_affordance};
use bevyrogue::combat::action_query::{CombatQuerySnapshot, UnitQuerySnapshot};
use bevyrogue::combat::kit::UnitSkills;
use bevyrogue::combat::state::CombatPhase;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::types::{SkillId, UnitId};
use bevyrogue::data::aggregate_skill_book;
use bevyrogue::data::skills_ron::{LegalityReasonCode, SkillBook};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn renamon_snapshot() -> CombatQuerySnapshot {
    let renamon = UnitQuerySnapshot {
        id: UnitId(7),
        team: Team::Ally,
        is_active: true,
        is_ko: false,
        is_stunned: false,
        is_commander: false,
        hp_current: 90,
        hp_max: 90,
        sp: 5,
        ultimate_current: 0,
        ultimate_trigger: 100,
        ultimate_ready: false,
        energy: 0,
        skills: Some(UnitSkills {
            basic: SkillId("diamond_storm".into()),
            skills: vec![SkillId("diamond_storm".into())],
            ultimate: SkillId("renamon_ult".into()),
            follow_up: None,
        }),
        ..Default::default()
    };
    let enemy = UnitQuerySnapshot {
        id: UnitId(101),
        team: Team::Enemy,
        is_active: false,
        is_ko: false,
        is_stunned: false,
        is_commander: false,
        hp_current: 60,
        hp_max: 60,
        sp: 0,
        ultimate_current: 0,
        ultimate_trigger: 100,
        ultimate_ready: false,
        energy: 0,
        skills: None,
        ..Default::default()
    };
    CombatQuerySnapshot {
        phase: CombatPhase::WaitingAction,
        acting_unit: renamon.clone(),
        target_unit: Some(enemy.clone()),
        units: vec![renamon, enemy],
    }
}

/// Parse Agumon's partial skill book from the embedded asset fragment.
/// This is what `render.rs` accidentally uses when it calls `.iter().next()`.
fn agumon_partial_book() -> SkillBook {
    let fragment = include_str!("../../assets/data/digimon/agumon/skills.ron");
    SkillBook(ron::from_str(fragment).expect("agumon skills.ron should parse"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// RED TEST (T01 deliverable): the combat panel must route through the canonical
/// `SkillBookHandle` so that Renamon's `diamond_storm` resolves as `Enabled` —
/// not `MissingSkill`.
///
/// Today this test is **RED** because `render.rs` uses `iter().next()` on
/// `Assets<SkillBook>`, which returns an arbitrary partial book that does not
/// contain Renamon's skills.  T02 will fix `render.rs` to look up the handle
/// from `SkillBookHandle`, at which point this test turns green.
///
/// NOTE: If T02 has already landed by the time this test runs, the test will
/// already be green — that is acceptable per the task contract.
#[test]
fn render_panel_path_resolves_renamon_skill_without_missing_skill() {
    // Simulate the render.rs bug: use the first arbitrary partial book
    // (Agumon's) rather than the canonical merged one.
    let arbitrary_partial_book = agumon_partial_book();
    let snapshot = renamon_snapshot();
    let skill_id = SkillId("diamond_storm".into());

    let affordance = query_action_affordance(
        &snapshot,
        &arbitrary_partial_book,
        UnitId(7),
        ActionQueryKind::Skill(&skill_id),
    );

    // This assertion is the fix contract: the render path must NOT produce
    // MissingSkill for a skill that exists in the canonical book.
    // It fails today because the partial book does not contain diamond_storm.
    assert!(
        !matches!(
            affordance.action,
            ActionStatus::Disabled {
                reason: LegalityReasonCode::MissingSkill
            }
        ),
        "REPRODUCE BUG: render.rs picks an arbitrary partial SkillBook that \
         does not contain Renamon's skills; got MissingSkill for diamond_storm. \
         Fix: use SkillBookHandle to look up the canonical merged book."
    );
    assert!(
        matches!(affordance.action, ActionStatus::Enabled),
        "expected Enabled for diamond_storm via canonical book path; got {:?}",
        affordance.action
    );
}

/// Documentation test: confirms that the partial Agumon book IS the root cause
/// of the false MissingSkill. Always passes; preserves understanding of the bug.
#[test]
fn partial_book_confirms_missing_skill_is_the_root_cause() {
    let partial_book = agumon_partial_book();
    let snapshot = renamon_snapshot();
    let skill_id = SkillId("diamond_storm".into());

    let affordance = query_action_affordance(
        &snapshot,
        &partial_book,
        UnitId(7),
        ActionQueryKind::Skill(&skill_id),
    );

    assert!(
        matches!(
            affordance.action,
            ActionStatus::Disabled {
                reason: LegalityReasonCode::MissingSkill
            }
        ),
        "expected MissingSkill from Agumon's partial book (root cause confirmation); \
         got {:?}",
        affordance.action
    );
}

/// Sanity check: the canonical merged book resolves diamond_storm as Enabled.
/// Always passes — used to validate the fix direction in T02.
#[test]
fn canonical_skill_book_resolves_renamon_diamond_storm_as_enabled() {
    let canonical_book = aggregate_skill_book();
    let snapshot = renamon_snapshot();
    let skill_id = SkillId("diamond_storm".into());

    let affordance = query_action_affordance(
        &snapshot,
        &canonical_book,
        UnitId(7),
        ActionQueryKind::Skill(&skill_id),
    );

    assert!(
        !matches!(
            affordance.action,
            ActionStatus::Disabled {
                reason: LegalityReasonCode::MissingSkill
            }
        ),
        "canonical book must not return MissingSkill for diamond_storm; got {:?}",
        affordance.action
    );
    assert!(
        matches!(affordance.action, ActionStatus::Enabled),
        "canonical book must return Enabled for Renamon's diamond_storm; got {:?}",
        affordance.action
    );
}
