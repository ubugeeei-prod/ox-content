use std::time::{Duration, Instant};

use super::super::{PmOptions, transform_pm, transform_pm_with};

#[test]
fn settled_blocks_agree_with_searching_afresh() {
    const PIECES: [&str; 12] = [
        "<pm>", "</pm>", "<PM>", "<pm/>", "<pm a=\"", "\"", "'", ">", "npm i a", "<pmx>", " ", "\n",
    ];
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    for round in 0..5000 {
        let pieces = 1 + round % 40;
        let mut html = String::new();
        for _ in 0..pieces {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            html.push_str(PIECES[(state % PIECES.len() as u64) as usize]);
        }
        let remembered = transform_pm_with(&html, 0, PmOptions::default(), true);
        let afresh = transform_pm_with(&html, 0, PmOptions::default(), false);
        assert_eq!(remembered.html, afresh.html, "for {html:?}");
        assert_eq!(remembered.group_count, afresh.group_count, "for {html:?}");
    }
}

#[test]
fn unclosed_blocks_cost_linear_time() {
    // Measured before at 64 KiB: `<pm>` repeated took 1.2 s and grew x10 for
    // every x4; `<pm ` repeated 1.0 s.
    let time = |html: &str| {
        (0..3)
            .map(|_| {
                let started = Instant::now();
                std::hint::black_box(transform_pm(html, 0, PmOptions::default()).html.len());
                started.elapsed()
            })
            .min()
            .unwrap_or(Duration::MAX)
    };
    for (unit, suffix) in [("<pm>", ""), ("<pm ", ""), ("<pm>npm i a", "</pm>")] {
        let build = |bytes: usize| format!("{}{suffix}", unit.repeat(bytes / unit.len()));
        let (small, large) = (build(16 * 1024), build(64 * 1024));
        let (small_time, large_time) = (time(&small), time(&large));
        // 4x the input: linear is about x4, quadratic about x16.
        assert!(
            large_time < small_time * 8 + Duration::from_millis(20),
            "{unit:?}: 64 KiB took {large_time:?} against {small_time:?} for 16 KiB"
        );
    }
}
