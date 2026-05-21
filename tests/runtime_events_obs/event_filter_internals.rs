//! Relocated from `src/combat/runtime/event_filter.rs` (R003 — no inline `mod tests` in src/).
//! Pure relocate: all touched symbols are already `pub`.

use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::runtime::event_filter::EventFilter;
use bevyrogue::combat::runtime::signal::Signal;
use bevyrogue::combat::runtime::{CastId, SignalPayload};
use bevyrogue::combat::types::UnitId;

#[test]
fn composite_filters_match_as_expected() {
    let combat = Signal::CombatEvent(CombatEvent {
        kind: CombatEventKind::UltimateUsed { unit_id: UnitId(7) },
        source: UnitId(7),
        target: UnitId(7),
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });
    let blueprint = Signal::Blueprint {
        owner: "kernel".to_string(),
        name: "ult_used".to_string(),
        payload: SignalPayload::UnitTarget(UnitId(7)),
        cast_id: CastId::ROOT,
    };

    let filter = EventFilter::all([
        EventFilter::any([
            EventFilter::combat(|event| {
                matches!(&event.kind, CombatEventKind::UltimateUsed { .. })
            }),
            EventFilter::blueprint("kernel", "ult_used"),
        ]),
        EventFilter::not(EventFilter::blueprint("kernel", "ult_used")),
    ]);

    assert!(filter.matches(&combat));
    assert!(!filter.matches(&blueprint));
}
