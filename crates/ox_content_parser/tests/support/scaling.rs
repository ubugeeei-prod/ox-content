//! Shared harness for tests that pin a parse back to linear time: build an
//! input at two sizes, time both, and require the ratio to stay well under
//! what a quadratic path would show.

#![allow(dead_code)]

use std::sync::{Mutex, PoisonError, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

/// Generous enough that a slow shared runner never trips it, and far below
/// what a quadratic path needs for these sizes.
const BUDGET: Duration = Duration::from_secs(30);

const SMALL: usize = 32 * 1024;
/// The large input is this many times the small one.
const FACTOR: usize = 4;

/// At least this many rounds are timed, and the best of each size kept.
const MIN_ROUNDS: usize = 3;
/// A ratio that still looks superlinear earns rounds up to this many before
/// it fails the build: noise washes out of the best times, a quadratic path
/// never does.
const MAX_ROUNDS: usize = 20;
/// A large parse this fast leaves no room for a quadratic path — the ones
/// these tests replaced took seconds at 128 KiB — and a ratio between such
/// small times is mostly scheduler noise on a busy runner.
const NOISE_FLOOR: Duration = Duration::from_millis(500);

/// One shape is timed at a time. The tests of a binary run on parallel
/// threads, and two of them timing at once would each see the other as load.
static TIMING: Mutex<()> = Mutex::new(());

/// One piece of a generated input: written once, or repeated to fill its
/// share of the requested size.
#[derive(Clone, Copy)]
pub enum Part {
    Once(&'static str),
    Fill(&'static str),
}

pub fn build(shape: &[Part], bytes: usize) -> String {
    let fills = shape.iter().filter(|part| matches!(part, Part::Fill(_))).count().max(1);
    let mut out = String::with_capacity(bytes + 64);
    for part in shape {
        match part {
            Part::Once(text) => out.push_str(text),
            Part::Fill(unit) => {
                let target = out.len() + bytes / fills;
                while out.len() < target {
                    out.push_str(unit);
                }
            }
        }
    }
    out
}

/// Time to parse `source` `times` over, into an arena that is reused so
/// that no run after the first pays for fresh pages.
fn time_parses(
    arena: &mut Allocator,
    source: &str,
    options: &ParserOptions,
    times: usize,
) -> Duration {
    let started = Instant::now();
    for _ in 0..times {
        arena.reset();
        let parsed = Parser::with_options(arena, source, options.clone()).parse().is_ok();
        assert!(parsed, "the input should parse to a document");
    }
    started.elapsed()
}

/// Requires every shape to cost about `FACTOR` times as much at `FACTOR`
/// times the size.
///
/// The parses run on a worker thread so a regression fails the suite in
/// bounded time instead of hanging it. Each round times the small input
/// `FACTOR` times over against the large one once: when the cost is linear
/// the two take equally long, so a runner that is busy throughout slows
/// both alike instead of taxing the longer measurement more.
pub fn assert_linear(options: &ParserOptions, shapes: &[&[Part]]) {
    for shape in shapes {
        let _turn = TIMING.lock().unwrap_or_else(PoisonError::into_inner);
        let small_source = build(shape, SMALL);
        let large_source = build(shape, SMALL * FACTOR);
        let options = options.clone();

        let (go, rounds) = mpsc::channel::<()>();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let (mut small_arena, mut large_arena) = (Allocator::new(), Allocator::new());
            while rounds.recv().is_ok() {
                let small = time_parses(&mut small_arena, &small_source, &options, FACTOR);
                let large = time_parses(&mut large_arena, &large_source, &options, 1);
                if sender.send((small, large)).is_err() {
                    break;
                }
            }
        });

        let (mut small, mut large) = (BUDGET, BUDGET);
        let mut ratio = f64::INFINITY;
        for round in 0..MAX_ROUNDS {
            go.send(()).expect("the worker waits for each round");
            let (small_round, large_round) =
                receiver.recv_timeout(BUDGET).expect("the input should parse in bounded time");
            small = small.min(small_round / FACTOR as u32);
            large = large.min(large_round);
            ratio = large.as_secs_f64() / small.as_secs_f64().max(1e-9);
            if round + 1 >= MIN_ROUNDS && (ratio < 8.0 || large < NOISE_FLOOR) {
                break;
            }
        }

        // 4x the input. Linear costs about 4x the time; quadratic costs 16x.
        let sample = build(shape, 40);
        assert!(
            ratio < 8.0 || large < NOISE_FLOOR,
            "{sample:?}...: 128 KiB took {large:?} against {small:?} for 32 KiB (x{ratio:.1}); \
             linear is about x4, quadratic about x16"
        );
    }
}

/// Deterministic xorshift, so a failure names an input that fails again.
pub struct Random(pub u64);

impl Random {
    pub fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % bound as u64) as usize
    }
}
