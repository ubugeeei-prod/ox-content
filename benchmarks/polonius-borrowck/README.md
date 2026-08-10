# Polonius Alpha borrow checker

[Polonius Alpha][blog] is the next iteration of the borrow checker. It became
the default on nightly in `nightly-2026-08-06`, and stabilization is planned
before the end of 2026, so it is expected to arrive on stable whether or not
this workspace is ready for it. This directory answers two questions ahead of
time: what it costs to compile, and what it lets us delete.

Run it with:

```bash
node benchmarks/polonius-borrowck/run.mjs \
  --toolchain nightly-2026-08-09 --iterations 4 --json results.json
```

`--toolchain` pins the compiler the results below were collected with; it
defaults to `nightly`, which measures whatever nightly is current instead.

The harness needs a nightly toolchain and a POSIX `/usr/bin/time`. It never
mutates the workspace beyond touching crate roots to force a re-check, and it
builds into throwaway target directories, so it will not disturb an existing
`target/`.

## What was measured

One toolchain, one flag difference: `-Zpolonius=off` selects today's stable NLL
borrow checker, `-Zpolonius=next` selects Polonius Alpha. Everything else — the
compiler build, the dependency graph, the machine — is held constant, so the
delta is attributable to the borrow checker alone.

Two phases, because they answer different questions:

- **`clean`** — a full `cargo check --workspace` from an empty target directory,
  dependencies included. This is the number a developer feels.
- **`borrowck`** — each of the 26 workspace crates re-checked on its own with
  `-Ztime-passes`, reporting the `MIR_borrow_checking` pass in isolation. One
  target per crate (its lib, or its bins when it has no lib), so this is where a
  borrow-checker change has to show up for library and binary code; examples,
  tests, and benches are not checked.

Two measurement details are load-bearing, and both were mistakes first:

- **CPU time, not wall clock, for the `clean` phase.** Wall clock on a laptop
  drifts with temperature by more than the effect being measured — an early run
  of this benchmark produced a "+11%" and a "−1.6%" from the same pair of
  configurations on the same machine an hour apart. CPU time (user + sys, over
  the whole `cargo` + `rustc` tree) is far steadier, and even it should be read
  as an upper bound on the noise rather than a precise figure.
- **One crate per `rustc` invocation for the `borrowck` phase.** Cargo replays
  the cached stderr of units it does not rebuild. If dependencies are built with
  `-Ztime-passes`, their timing lines reappear on every later run and get
  counted against whichever crate was being checked, which inflated an early
  measurement of this pass by roughly 7×. Passing the flag through
  `cargo rustc -p <crate> -- -Ztime-passes` means no dependency ever emits a
  timing line at all.

## Results

`rustc 1.99.0-nightly (969b803cb 2026-08-09)`, Apple M2 Max (12 cores, 96 GB),
macOS 26.5.1, 4 iterations per configuration in ABBA order.

| Measurement                                     |           NLL | Polonius Alpha |     Delta |
| ----------------------------------------------- | ------------: | -------------: | --------: |
| `MIR_borrow_checking`, 26 crates, min per crate |        1.50 s |         1.62 s | **+8.0%** |
| Clean workspace check, CPU time                 | 108.3 s ± 8.5 |  111.0 s ± 6.1 |     +2.4% |
| Clean workspace check, wall clock               |  17.7 s ± 1.8 |   17.3 s ± 1.4 |     −1.9% |

The borrow-checking pass itself is consistently slower, and consistently in one
direction: 21 of 26 crates regressed, 5 were unchanged, none improved. The
largest movers are the crates with the most borrow-heavy control flow.

| Crate               |     NLL | Polonius Alpha |  Delta |
| ------------------- | ------: | -------------: | -----: |
| `ox_content_docs`   | 0.373 s |        0.411 s | +10.2% |
| `ox_content_ssg`    | 0.166 s |        0.187 s | +12.7% |
| `ox_content_napi`   | 0.250 s |        0.262 s |  +4.8% |
| `ox_content_parser` | 0.091 s |        0.101 s | +11.0% |
| `ox_content_lsp`    | 0.184 s |        0.194 s |  +5.4% |

That 8% lands on a pass that is a small fraction of a compile. In absolute terms
it is 0.12 s spread across the whole workspace, which is why it does not survive
into the `clean` numbers: at workspace scale the difference sits inside the
run-to-run spread, and repeated rounds put it between +0.7% and +2.4% CPU with
wall clock unable to distinguish the two at all. The "relatively few and
minimal" regressions the release announcement describes are a statement about
magnitude, and by that measure these qualify: the regression is broad here (21
of 26 crates), but every crate moves by a fraction of a second, the workspace
total is unchanged, and nothing comes near the 2–3× worst case the announcement
reports for borrow-heavy crates.

**Nothing in the workspace fails to compile under Polonius Alpha**, with no new
warnings — checked across all 26 crates.

## Borrow-checker workarounds this would let us delete

Polonius Alpha accepts code NLL rejects, so some existing code is shaped around
a restriction that is going away. Each candidate below was verified by actually
rewriting it and compiling the result both ways — `-Zpolonius=off` rejects it,
`-Zpolonius=next` accepts it.

All three are the same shape: a function that takes `&'a mut` a cache and
returns a borrow out of it. The natural spelling — look up, return the hit,
otherwise insert and return — is [NLL problem case #3][case3]. NLL rejects it
because the borrow taken for the early return is held across the later `insert`,
even on the path where the early return did not happen.

- **`normalized_entries_for_module`** —
  [`crates/ox_content_docs/src/graph/docs.rs:76`](../../crates/ox_content_docs/src/graph/docs.rs#L76).
  Guards with `contains_key`, then repeats the lookup as `get(...).expect(...)`,
  costing a second hash of every path on every cache hit. Self-documenting: the
  comment above it already says the shape exists to "keep the returned borrow
  simple".
- **`get_transformed_file`** —
  [`crates/ox_content_napi/src/collection_bindings.rs:207`](../../crates/ox_content_napi/src/collection_bindings.rs#L207).
  The same double lookup. Its `expect("transformed file should be cached")`
  documents an invariant that exists only because the borrow could not be held
  across the insert.
- **`get_prepared_file`** —
  [`crates/ox_content_napi/src/collection_bindings.rs:181`](../../crates/ox_content_napi/src/collection_bindings.rs#L181).
  Sidesteps the problem by returning an owned `PreparedFile` rather than a
  borrow, which costs two clones of the struct per call. This is the most
  valuable of the three to fix, since returning `&'a PreparedFile` removes real
  copying rather than one hash lookup.

### Why these are not fixed in this PR

Applying them would break the build for everyone. The workspace declares
`rust-version = "1.83.0"` and CI pins stable; `-Zpolonius=next` is nightly-only
until stabilization lands. Each rewrite above compiles _only_ under Polonius,
so committing it would take the crate from "builds on stable" to "builds on
nightly with a `-Z` flag". They are recorded here so the work is already scoped
when Polonius reaches stable, and so nobody re-derives the same three sites.

Two further candidates were investigated and rejected, which is worth recording
so they are not re-examined:

- `ox_content_ssg/src/assets/chunk.rs:35` `get_or_create` stores an index rather
  than holding a borrow, but the simplified form compiles under NLL too — the
  `Vec` it indexes is load-bearing for deterministic output order, not a
  borrow-checker concession.
- The block scope around `module_entries` in
  `ox_content_docs/src/graph/entrypoint_docs.rs:82` also compiles without the
  braces under NLL. It is stylistic.

Disjoint-field borrow accommodations elsewhere in the tree (for example the
`&self.doc_extractor` / `&mut self.docs` split in `graph/builder.rs`) are a
different limitation — view types — which Polonius Alpha does not address.

## If Polonius Alpha causes trouble

It can be turned off without leaving nightly:

```bash
RUSTFLAGS=-Zpolonius=off cargo build
```

Regressions and unsoundness go to the [tracking issue][tracking].

[blog]: https://blog.rust-lang.org/2026/08/04/enabling-polonius-alpha-on-nightly/
[case3]: https://rust-lang.github.io/rfcs/2094-nll.html#problem-case-3-conditional-control-flow-across-functions
[tracking]: https://github.com/rust-lang/rust/issues/160456
