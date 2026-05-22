#![cfg(feature = "windowed")]

//! Headless proof for the windowed frame-time soak surface (S09 T02).
//!
//! Reads no `.gsd` artifact. Exercises the same pure lib surface the windowed
//! soak wires into `WindowedValidationState`: the `BEVYROGUE_VALIDATION_BASELINE`
//! toggle parser and the `FrameTimeAccumulator` that the presentation tick feeds
//! `Time::delta_secs()` into, plus the structured `validation_frametime:` line.

use bevyrogue::combat::observability::{
    FrameTimeAccumulator, format_frame_time_stats, parse_validation_baseline_toggle,
};

#[test]
fn frame_time_baseline_toggle_maps_documented_truthy_and_falsey_strings() {
    // Unset → false: the default soak is a full anim-graph run.
    assert_eq!(parse_validation_baseline_toggle(None), Ok(false));

    for falsey in ["0", "false", "False", "FALSE", "no", "No", "NO", "off"] {
        assert_eq!(
            parse_validation_baseline_toggle(Some(falsey)),
            Ok(false),
            "{falsey} should map to false"
        );
    }

    // Empty string counts as truthy (presence of the var), matching the windowed
    // validation toggle convention.
    for truthy in ["", "1", "true", "True", "TRUE", "yes", "Yes", "YES", "on"] {
        assert_eq!(
            parse_validation_baseline_toggle(Some(truthy)),
            Ok(true),
            "{truthy:?} should map to true"
        );
    }

    assert!(
        parse_validation_baseline_toggle(Some("maybe")).is_err(),
        "undocumented values must be a hard error"
    );
}

#[test]
fn frame_time_known_delta_series_yields_expected_soak_stats_and_line() {
    // Mirrors the accumulator wired into WindowedValidationState::record_frame:
    // feed a known per-frame delta series (seconds) and assert the finalised stats
    // and the structured line the soak emits at finish for both modes.
    let mut acc = FrameTimeAccumulator::default();
    acc.push(0.100); // one slow 100 ms frame, pushed out of order
    for _ in 0..19 {
        acc.push(0.010); // nineteen 10 ms frames
    }
    let stats = acc.finalise();

    assert_eq!(stats.count, 20);
    assert!((stats.min_ms - 10.0).abs() < 0.5, "min_ms={}", stats.min_ms);
    assert!(
        (stats.max_ms - 100.0).abs() < 0.5,
        "max_ms={}",
        stats.max_ms
    );
    // mean = (19*10 + 100) / 20 = 290 / 20 = 14.5 ms
    assert!(
        (stats.mean_ms - 14.5).abs() < 0.1,
        "mean_ms={}",
        stats.mean_ms
    );
    // p95 index = ceil(0.95 * 20) - 1 = 18 → sorted[18] = 10 ms
    assert!((stats.p95_ms - 10.0).abs() < 0.5, "p95_ms={}", stats.p95_ms);

    let full = format_frame_time_stats(&stats, "full");
    assert!(full.starts_with("validation_frametime:"), "{full}");
    assert!(full.contains("mode=full"), "{full}");
    assert!(full.contains("count=20"), "{full}");
    assert!(full.contains("mean_ms=14.500"), "{full}");

    let baseline = format_frame_time_stats(&stats, "baseline");
    assert!(baseline.contains("mode=baseline"), "{baseline}");
}
