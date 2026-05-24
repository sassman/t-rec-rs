use std::path::PathBuf;
use std::time::Duration;

use crate::core::capture::{FrameRecord, Recording};

/// One display segment: an image shown for a duration.
/// Renderer adapters convert this into format-specific directives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub path: PathBuf,
    pub duration_ms: u128,
}

/// Walk the timeline, collapse Skipped runs onto their preceding Frame, apply idle cap.
///
/// Returns segments in playback order. Empty input → empty output.
pub fn timeline_to_segments(
    recording: &Recording,
    idle_cap: Option<Duration>,
) -> Vec<Segment> {
    let end_ms = recording.end_ms.unwrap_or_else(|| {
        recording.frames.last().map(tc_of).unwrap_or(0)
    });
    let cap_ms = idle_cap.map(|d| d.as_millis());

    let mut segments = Vec::new();
    let mut current: Option<(PathBuf, u128)> = None;

    for entry in &recording.frames {
        match entry {
            FrameRecord::Frame { tc_ms, path } => {
                if let Some((prev_path, prev_start)) = current.take() {
                    segments.push(make_segment(prev_path, prev_start, *tc_ms, cap_ms));
                }
                current = Some((path.clone(), *tc_ms));
            }
            FrameRecord::Skipped { .. } => { /* extends current segment via end_ms below */ }
        }
    }
    if let Some((path, start)) = current {
        segments.push(make_segment(path, start, end_ms, cap_ms));
    }
    segments
}

fn make_segment(path: PathBuf, start_tc: u128, end_tc: u128, cap_ms: Option<u128>) -> Segment {
    let real_span = end_tc.saturating_sub(start_tc);
    let duration_ms = match cap_ms {
        Some(cap) => real_span.min(cap),
        None => real_span,
    };
    Segment { path, duration_ms }
}

fn tc_of(entry: &FrameRecord) -> u128 {
    match entry {
        FrameRecord::Frame { tc_ms, .. } | FrameRecord::Skipped { tc_ms } => *tc_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_recording_yields_no_segments() {
        let r = Recording::default();
        assert_eq!(timeline_to_segments(&r, None), Vec::<Segment>::new());
    }

    #[test]
    fn single_frame_spans_to_end_ms() {
        let mut r = Recording::default();
        r.push(FrameRecord::Frame { tc_ms: 100, path: PathBuf::from("a.bmp") });
        r.finish(500);

        let segments = timeline_to_segments(&r, None);
        assert_eq!(segments, vec![Segment {
            path: PathBuf::from("a.bmp"),
            duration_ms: 400,  // 500 (end) - 100 (start)
        }]);
    }

    #[test]
    fn skipped_runs_extend_the_preceding_frame_segment() {
        let mut r = Recording::default();
        r.push(FrameRecord::Frame { tc_ms: 0, path: PathBuf::from("a.bmp") });
        r.push(FrameRecord::Skipped { tc_ms: 250 });
        r.push(FrameRecord::Skipped { tc_ms: 500 });
        r.push(FrameRecord::Frame { tc_ms: 750, path: PathBuf::from("b.bmp") });
        r.finish(1000);

        let segments = timeline_to_segments(&r, None);
        assert_eq!(segments, vec![
            Segment { path: PathBuf::from("a.bmp"), duration_ms: 750 },  // 750-0 (next Frame)
            Segment { path: PathBuf::from("b.bmp"), duration_ms: 250 },  // 1000-750 (end_ms)
        ]);
    }

    #[test]
    fn last_segment_uses_end_ms_not_carry_forward() {
        let mut r = Recording::default();
        r.push(FrameRecord::Frame { tc_ms: 0, path: PathBuf::from("a.bmp") });
        r.push(FrameRecord::Frame { tc_ms: 250, path: PathBuf::from("b.bmp") });
        r.finish(10_000);   // long tail after last Frame

        let segments = timeline_to_segments(&r, None);
        assert_eq!(segments[1].duration_ms, 9_750);  // 10000 - 250, not 250 (the previous delta)
    }

    #[test]
    fn no_first_frame_flash_when_two_frames_have_equal_intervals() {
        let mut r = Recording::default();
        r.push(FrameRecord::Frame { tc_ms: 0, path: PathBuf::from("a.bmp") });
        r.push(FrameRecord::Frame { tc_ms: 250, path: PathBuf::from("b.bmp") });
        r.finish(500);

        let segments = timeline_to_segments(&r, None);
        assert_eq!(segments[0].duration_ms, 250);  // NOT 0 or 1, which would flash
    }

    #[test]
    fn idle_cap_none_preserves_real_spans() {
        let mut r = Recording::default();
        r.push(FrameRecord::Frame { tc_ms: 0, path: PathBuf::from("a.bmp") });
        r.push(FrameRecord::Skipped { tc_ms: 1_000 });
        r.push(FrameRecord::Skipped { tc_ms: 2_000 });
        r.push(FrameRecord::Frame { tc_ms: 5_000, path: PathBuf::from("b.bmp") });
        r.finish(6_000);

        let segments = timeline_to_segments(&r, None);
        assert_eq!(segments[0].duration_ms, 5_000);  // full 5s preserved
        assert_eq!(segments[1].duration_ms, 1_000);
    }

    #[test]
    fn idle_cap_some_caps_each_segment_independently() {
        let mut r = Recording::default();
        r.push(FrameRecord::Frame { tc_ms: 0, path: PathBuf::from("a.bmp") });
        r.push(FrameRecord::Skipped { tc_ms: 250 });
        r.push(FrameRecord::Frame { tc_ms: 5_000, path: PathBuf::from("b.bmp") });
        r.push(FrameRecord::Skipped { tc_ms: 5_250 });
        r.finish(10_000);

        let segments = timeline_to_segments(&r, Some(Duration::from_millis(3_000)));
        assert_eq!(segments[0].duration_ms, 3_000);  // 5000 capped to 3000
        assert_eq!(segments[1].duration_ms, 3_000);  // 5000 capped to 3000
    }

    #[test]
    fn missing_end_ms_falls_back_to_last_tc() {
        let mut r = Recording::default();
        r.push(FrameRecord::Frame { tc_ms: 0, path: PathBuf::from("a.bmp") });
        r.push(FrameRecord::Frame { tc_ms: 250, path: PathBuf::from("b.bmp") });
        // no finish() — end_ms stays None

        let segments = timeline_to_segments(&r, None);
        assert_eq!(segments[0].duration_ms, 250);
        assert_eq!(segments[1].duration_ms, 0);  // last tc - last tc
    }

    #[test]
    fn idle_cap_zero_does_not_panic() {
        let mut r = Recording::default();
        r.push(FrameRecord::Frame { tc_ms: 0, path: PathBuf::from("a.bmp") });
        r.push(FrameRecord::Frame { tc_ms: 1_000, path: PathBuf::from("b.bmp") });
        r.finish(2_000);

        let segments = timeline_to_segments(&r, Some(Duration::ZERO));
        assert!(segments.iter().all(|s| s.duration_ms == 0));
    }

    #[test]
    fn only_skipped_entries_yields_no_segments() {
        let mut r = Recording::default();
        r.push(FrameRecord::Skipped { tc_ms: 0 });
        r.push(FrameRecord::Skipped { tc_ms: 250 });
        r.finish(500);

        let segments = timeline_to_segments(&r, None);
        assert!(segments.is_empty());
    }

    #[test]
    fn regression_first_frame_does_not_flash() {
        // Pre-fix behavior: frame[0] received delay = tc[0] - 0 ≈ 0 (clamped to 1cs / 10ms),
        // so the first frame appeared for ~10ms before the second frame replaced it.
        // Fix: frame[0]'s display duration is the span to the next frame.
        let mut r = Recording::default();
        r.push(FrameRecord::Frame { tc_ms: 0, path: PathBuf::from("a.bmp") });
        r.push(FrameRecord::Frame { tc_ms: 1000, path: PathBuf::from("b.bmp") });
        r.finish(2000);

        let segments = timeline_to_segments(&r, None);
        assert_eq!(segments[0].duration_ms, 1000, "first frame must not flash");
    }

    #[test]
    fn regression_trailing_idle_after_last_frame_preserved() {
        // Pre-fix behavior: the time between the last captured Frame and Ctrl+D was lost
        // because the capture thread never recorded an end timestamp. For idle-heavy
        // recordings this dropped seconds off the gif.
        // Fix: end_ms is stamped on Stop; the last segment spans real_end - last_tc.
        let mut r = Recording::default();
        r.push(FrameRecord::Frame { tc_ms: 0, path: PathBuf::from("a.bmp") });
        r.push(FrameRecord::Frame { tc_ms: 250, path: PathBuf::from("b.bmp") });
        r.push(FrameRecord::Skipped { tc_ms: 500 });
        r.push(FrameRecord::Skipped { tc_ms: 750 });
        r.push(FrameRecord::Skipped { tc_ms: 1000 });
        r.finish(1200);

        let segments = timeline_to_segments(&r, None);
        // Last segment must show frame B from tc=250 to end=1200 = 950ms, not just to the last Skipped tc.
        assert_eq!(segments[1].duration_ms, 950);
    }

    #[test]
    fn regression_natural_mode_visibly_differs_from_dedup_on_idle() {
        // Issue #257: --natural / -n had no observable effect for the user's repro.
        // Capture is identical between modes when typing fast (every poll is unique);
        // the meaningful difference shows up on idle. This test asserts that a recording
        // with the same captures (one early change, then a long held state) renders
        // differently under natural mode (no cap) vs default (idle_pause=3s).

        let build = |dedup: bool| -> Recording {
            let mut r = Recording::default();
            r.push(FrameRecord::Frame { tc_ms: 0, path: PathBuf::from("a.bmp") });
            if dedup {
                r.push(FrameRecord::Skipped { tc_ms: 5_000 });
            } else {
                r.push(FrameRecord::Frame { tc_ms: 5_000, path: PathBuf::from("a.bmp") });
            }
            r.finish(10_000);
            r
        };

        let natural = timeline_to_segments(&build(false), None);
        let default = timeline_to_segments(&build(true), Some(Duration::from_secs(3)));

        // Natural mode: each segment plays for its real interval.
        assert_eq!(natural[0].duration_ms, 5_000);
        assert_eq!(natural[1].duration_ms, 5_000);
        // Default mode: idle is capped at 3s.
        assert_eq!(default[0].duration_ms, 3_000);
        // The two outputs must differ — that is the observable effect of -n.
        assert_ne!(natural[0].duration_ms, default[0].duration_ms);
    }
}
