/// Pure frame-time aggregator and regression comparator.
///
/// No Bevy types, no wall-clock reads, no RNG.  The caller (presentation
/// tick) supplies per-frame deltas in seconds.  All maths are deterministic
/// so the suite can run headlessly (R004).
///
/// D027 regression thresholds:
///   - mean regression <= 15 %  OR mean absolute delta <= 2 ms  (grace floor)
///   - p95  regression <= 20 %

// ── Accumulator ──────────────────────────────────────────────────────────────

/// Accumulates per-frame deltas (seconds) from the caller.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FrameTimeAccumulator {
    samples: Vec<f32>,
}

impl FrameTimeAccumulator {
    /// Push one frame delta (seconds).
    #[inline]
    pub fn push(&mut self, delta_secs: f32) {
        self.samples.push(delta_secs);
    }

    /// Number of samples collected.
    #[inline]
    pub fn count(&self) -> usize {
        self.samples.len()
    }

    /// Finalise into [`FrameTimeStats`].
    ///
    /// Returns zeroed stats when no samples have been collected.
    pub fn finalise(&self) -> FrameTimeStats {
        let count = self.samples.len();
        if count == 0 {
            return FrameTimeStats {
                count: 0,
                mean_ms: 0.0,
                p95_ms: 0.0,
                max_ms: 0.0,
                min_ms: 0.0,
            };
        }

        let mut sorted = self.samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let sum: f32 = sorted.iter().sum();
        let mean_ms = (sum / count as f32) as f64 * 1000.0;
        let min_ms = sorted[0] as f64 * 1000.0;
        let max_ms = sorted[count - 1] as f64 * 1000.0;

        // p95 index = ceil(0.95 * N) - 1, clamped to valid range.
        let p95_idx = ((0.95_f64 * count as f64).ceil() as usize)
            .saturating_sub(1)
            .min(count - 1);
        let p95_ms = sorted[p95_idx] as f64 * 1000.0;

        FrameTimeStats {
            count,
            mean_ms,
            p95_ms,
            max_ms,
            min_ms,
        }
    }
}

// ── Stats ─────────────────────────────────────────────────────────────────────

/// Aggregated frame-time statistics (all duration values in milliseconds).
#[derive(Debug, Clone)]
pub struct FrameTimeStats {
    /// Number of samples.
    pub count: usize,
    /// Arithmetic mean frame time in ms.
    pub mean_ms: f64,
    /// 95th-percentile frame time in ms.
    pub p95_ms: f64,
    /// Maximum frame time in ms.
    pub max_ms: f64,
    /// Minimum frame time in ms.
    pub min_ms: f64,
}

// ── Formatter ────────────────────────────────────────────────────────────────

/// Emit a `validation_frametime:` line for the soak log.
///
/// Format mirrors the `validation_snapshot:` prefix convention used by
/// [`crate::combat::observability::format_validation_snapshot`].
///
/// Example:
/// ```text
/// validation_frametime: mode=full count=1000 mean_ms=5.120 p95_ms=8.330 max_ms=12.400 min_ms=3.010
/// ```
pub fn format_frame_time_stats(stats: &FrameTimeStats, mode: &str) -> String {
    format!(
        "validation_frametime: mode={} count={} mean_ms={:.3} p95_ms={:.3} max_ms={:.3} min_ms={:.3}",
        mode, stats.count, stats.mean_ms, stats.p95_ms, stats.max_ms, stats.min_ms
    )
}

// ── Baseline run toggle ────────────────────────────────────────────────────────

/// Parse the `BEVYROGUE_VALIDATION_BASELINE` env toggle (kernel-only baseline run).
///
/// Lives in the lib (not the windowed binary) so it is headlessly testable. The
/// mapping mirrors the windowed validation toggle: unset → false (the default is a
/// full anim-graph run); the documented truthy strings → true; the documented
/// falsey strings → false; anything else is a hard error. Presence of the var with
/// an empty value counts as truthy, matching `BEVYROGUE_VALIDATION_WINDOWED`.
pub fn parse_validation_baseline_toggle(raw: Option<&str>) -> Result<bool, String> {
    match raw {
        None | Some("0" | "false" | "False" | "FALSE" | "no" | "No" | "NO" | "off") => Ok(false),
        Some("") | Some("1" | "true" | "True" | "TRUE" | "yes" | "Yes" | "YES" | "on") => Ok(true),
        Some(other) => Err(format!(
            "BEVYROGUE_VALIDATION_BASELINE must be one of: 1,true,yes,on,0,false,no,off (got {other:?})"
        )),
    }
}

// ── Regression verdict ───────────────────────────────────────────────────────

/// Outcome of a frame-time regression comparison (D027 thresholds).
///
/// `PartialEq` is implemented manually because `f64` is not `Eq`; comparisons
/// use exact bit equality (appropriate for tests that construct known values).
#[derive(Debug, Clone)]
pub enum RegressionVerdict {
    /// No regression detected — all thresholds passed.
    Pass,
    /// Mean regression exceeds 15 % AND the absolute delta exceeds 2 ms.
    MeanRegression {
        full_mean_ms: f64,
        baseline_mean_ms: f64,
        /// Percentage increase (positive = regression).
        pct_increase: f64,
    },
    /// p95 regression exceeds 20 %.
    P95Regression {
        full_p95_ms: f64,
        baseline_p95_ms: f64,
        /// Percentage increase (positive = regression).
        pct_increase: f64,
    },
    /// Both mean and p95 regressions detected.
    BothRegression { mean_pct: f64, p95_pct: f64 },
}

impl PartialEq for RegressionVerdict {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Pass, Self::Pass) => true,
            (
                Self::MeanRegression {
                    full_mean_ms: a1,
                    baseline_mean_ms: b1,
                    pct_increase: c1,
                },
                Self::MeanRegression {
                    full_mean_ms: a2,
                    baseline_mean_ms: b2,
                    pct_increase: c2,
                },
            ) => {
                a1.to_bits() == a2.to_bits()
                    && b1.to_bits() == b2.to_bits()
                    && c1.to_bits() == c2.to_bits()
            }
            (
                Self::P95Regression {
                    full_p95_ms: a1,
                    baseline_p95_ms: b1,
                    pct_increase: c1,
                },
                Self::P95Regression {
                    full_p95_ms: a2,
                    baseline_p95_ms: b2,
                    pct_increase: c2,
                },
            ) => {
                a1.to_bits() == a2.to_bits()
                    && b1.to_bits() == b2.to_bits()
                    && c1.to_bits() == c2.to_bits()
            }
            (
                Self::BothRegression {
                    mean_pct: a1,
                    p95_pct: b1,
                },
                Self::BothRegression {
                    mean_pct: a2,
                    p95_pct: b2,
                },
            ) => a1.to_bits() == a2.to_bits() && b1.to_bits() == b2.to_bits(),
            _ => false,
        }
    }
}

/// Compare `full` (anim-graph) stats against `baseline` (kernel-only) stats
/// using the D027 thresholds:
///   - mean regression > 15 %  AND mean absolute delta > 2 ms  → fail
///   - p95  regression > 20 %                                   → fail
pub fn frame_time_regression(
    full: &FrameTimeStats,
    baseline: &FrameTimeStats,
) -> RegressionVerdict {
    // If baseline is zero we cannot compute a ratio — treat as pass.
    let mean_fail = if baseline.mean_ms > 0.0 {
        let pct = (full.mean_ms - baseline.mean_ms) / baseline.mean_ms * 100.0;
        let abs_delta = full.mean_ms - baseline.mean_ms;
        // Fail only when BOTH the relative threshold AND the absolute tolerance
        // are exceeded (the 2 ms floor prevents false positives on fast baselines).
        pct > 15.0 && abs_delta > 2.0
    } else {
        false
    };

    let p95_fail = if baseline.p95_ms > 0.0 {
        let pct = (full.p95_ms - baseline.p95_ms) / baseline.p95_ms * 100.0;
        pct > 20.0
    } else {
        false
    };

    match (mean_fail, p95_fail) {
        (false, false) => RegressionVerdict::Pass,
        (true, false) => {
            let pct = (full.mean_ms - baseline.mean_ms) / baseline.mean_ms * 100.0;
            RegressionVerdict::MeanRegression {
                full_mean_ms: full.mean_ms,
                baseline_mean_ms: baseline.mean_ms,
                pct_increase: pct,
            }
        }
        (false, true) => {
            let pct = (full.p95_ms - baseline.p95_ms) / baseline.p95_ms * 100.0;
            RegressionVerdict::P95Regression {
                full_p95_ms: full.p95_ms,
                baseline_p95_ms: baseline.p95_ms,
                pct_increase: pct,
            }
        }
        (true, true) => {
            let mean_pct = (full.mean_ms - baseline.mean_ms) / baseline.mean_ms * 100.0;
            let p95_pct = (full.p95_ms - baseline.p95_ms) / baseline.p95_ms * 100.0;
            RegressionVerdict::BothRegression { mean_pct, p95_pct }
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Accumulator edge-cases ────────────────────────────────────────────────

    #[test]
    fn empty_accumulator_returns_zeroed_stats() {
        let acc = FrameTimeAccumulator::default();
        assert_eq!(acc.count(), 0);
        let stats = acc.finalise();
        assert_eq!(stats.count, 0);
        assert_eq!(stats.mean_ms, 0.0);
        assert_eq!(stats.p95_ms, 0.0);
        assert_eq!(stats.max_ms, 0.0);
        assert_eq!(stats.min_ms, 0.0);
    }

    #[test]
    fn single_sample() {
        let mut acc = FrameTimeAccumulator::default();
        acc.push(0.016); // 16 ms
        assert_eq!(acc.count(), 1);
        let stats = acc.finalise();
        assert_eq!(stats.count, 1);
        assert!(
            (stats.mean_ms - 16.0).abs() < 0.01,
            "mean_ms={}",
            stats.mean_ms
        );
        assert!(
            (stats.p95_ms - 16.0).abs() < 0.01,
            "p95_ms={}",
            stats.p95_ms
        );
        assert!(
            (stats.max_ms - 16.0).abs() < 0.01,
            "max_ms={}",
            stats.max_ms
        );
        assert!(
            (stats.min_ms - 16.0).abs() < 0.01,
            "min_ms={}",
            stats.min_ms
        );
    }

    // ── p95 with unsorted series ──────────────────────────────────────────────

    #[test]
    fn p95_with_unsorted_series() {
        // 20 samples: 19 × 10 ms + 1 × 100 ms pushed in non-sorted order.
        let mut acc = FrameTimeAccumulator::default();
        acc.push(0.100); // 100 ms — pushed first (unsorted)
        for _ in 0..19 {
            acc.push(0.010); // 10 ms
        }
        assert_eq!(acc.count(), 20);
        let stats = acc.finalise();

        // p95 index = ceil(0.95 * 20) - 1 = ceil(19.0) - 1 = 18
        // sorted[0..18] = 10 ms, sorted[19] = 100 ms
        // → p95 = sorted[18] = 10 ms
        assert!(
            (stats.p95_ms - 10.0).abs() < 0.5,
            "expected p95 ≈ 10 ms, got {:.3}",
            stats.p95_ms
        );
        assert!(
            (stats.max_ms - 100.0).abs() < 0.5,
            "max_ms={:.3}",
            stats.max_ms
        );
        assert!(
            (stats.min_ms - 10.0).abs() < 0.5,
            "min_ms={:.3}",
            stats.min_ms
        );
    }

    // ── Regression verdicts ───────────────────────────────────────────────────

    fn make_stats(mean_ms: f64, p95_ms: f64) -> FrameTimeStats {
        FrameTimeStats {
            count: 100,
            mean_ms,
            p95_ms,
            max_ms: p95_ms * 1.1,
            min_ms: mean_ms * 0.5,
        }
    }

    #[test]
    fn verdict_pass_when_no_regression() {
        let baseline = make_stats(10.0, 15.0);
        let full = make_stats(10.5, 15.5); // +5 % mean, +3.3 % p95
        assert_eq!(
            frame_time_regression(&full, &baseline),
            RegressionVerdict::Pass
        );
    }

    #[test]
    fn verdict_mean_regression_at_2ms_boundary() {
        let baseline = make_stats(10.0, 15.0);
        // +20 % relative but only +2.0 ms absolute — NOT > 2.0, should pass.
        let full_edge = make_stats(12.0, 16.0);
        assert_eq!(
            frame_time_regression(&full_edge, &baseline),
            RegressionVerdict::Pass,
            "exact 2 ms delta should be absorbed by the tolerance floor"
        );

        // +30 % relative AND +3 ms absolute → should fail mean.
        let full_fail = make_stats(13.0, 16.0);
        assert!(
            matches!(
                frame_time_regression(&full_fail, &baseline),
                RegressionVerdict::MeanRegression { .. }
            ),
            "expected MeanRegression"
        );
    }

    #[test]
    fn verdict_p95_regression() {
        let baseline = make_stats(10.0, 10.0);
        // p95 +26 % (> 20 %), mean +5 % within tolerance.
        let full = make_stats(10.5, 12.6);
        assert!(
            matches!(
                frame_time_regression(&full, &baseline),
                RegressionVerdict::P95Regression { .. }
            ),
            "expected P95Regression"
        );
    }

    #[test]
    fn verdict_both_regression() {
        let baseline = make_stats(10.0, 10.0);
        // mean +30 % AND +3 ms absolute, p95 +26 %.
        let full = make_stats(13.0, 12.6);
        assert!(
            matches!(
                frame_time_regression(&full, &baseline),
                RegressionVerdict::BothRegression { .. }
            ),
            "expected BothRegression"
        );
    }

    // ── 2 ms absolute tolerance edge ─────────────────────────────────────────

    #[test]
    fn absolute_2ms_tolerance_absorbs_large_relative_on_fast_baseline() {
        // Baseline is very fast (1 ms mean); a 50 % relative regression is only
        // 0.5 ms absolute → the 2 ms tolerance floor must protect it.
        let baseline = make_stats(1.0, 1.5);
        let full = make_stats(1.5, 1.7); // +50 % relative, +0.5 ms absolute
        assert_eq!(
            frame_time_regression(&full, &baseline),
            RegressionVerdict::Pass,
            "small absolute delta should be absorbed even if relative pct is high"
        );
    }

    // ── Formatter ────────────────────────────────────────────────────────────

    // ── Baseline toggle ────────────────────────────────────────────────────────

    #[test]
    fn baseline_toggle_maps_documented_strings_and_rejects_garbage() {
        assert_eq!(parse_validation_baseline_toggle(None), Ok(false));
        for falsey in ["0", "false", "False", "FALSE", "no", "No", "NO", "off"] {
            assert_eq!(
                parse_validation_baseline_toggle(Some(falsey)),
                Ok(false),
                "{falsey}"
            );
        }
        for truthy in ["", "1", "true", "True", "TRUE", "yes", "Yes", "YES", "on"] {
            assert_eq!(
                parse_validation_baseline_toggle(Some(truthy)),
                Ok(true),
                "{truthy}"
            );
        }
        assert!(parse_validation_baseline_toggle(Some("maybe")).is_err());
    }

    #[test]
    fn format_starts_with_validation_frametime_prefix() {
        let stats = make_stats(5.0, 8.0);
        let line = format_frame_time_stats(&stats, "full");
        assert!(
            line.starts_with("validation_frametime:"),
            "must start with validation_frametime: prefix, got: {line}"
        );
        assert!(line.contains("mode=full"), "got: {line}");
        assert!(line.contains("mean_ms=5.000"), "got: {line}");
        assert!(line.contains("p95_ms=8.000"), "got: {line}");
        assert!(line.contains("count=100"), "got: {line}");
    }
}
