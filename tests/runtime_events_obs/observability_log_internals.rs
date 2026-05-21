use bevyrogue::combat::observability::log::{ActionLog, LogEntry};
use bevyrogue::combat::types::UnitId;

#[test]
fn action_log_caps_at_5() {
    let mut log = ActionLog::default();
    for i in 0..7u32 {
        log.push(LogEntry::Ko { target: UnitId(i) });
    }
    assert_eq!(log.events.len(), 5);
    // Oldest two (UnitId(0), UnitId(1)) must be evicted
    if let Some(LogEntry::Ko { target }) = log.events.front() {
        assert_eq!(*target, UnitId(2));
    } else {
        panic!("expected Ko event at front");
    }
}
