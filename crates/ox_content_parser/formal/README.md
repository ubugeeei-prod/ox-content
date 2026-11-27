# Formal proofs for `ox_content_parser`

The parser's scaling tests (`tests/*_scaling.rs`) measure that a parse
grows linearly with its input on the shapes that used to be quadratic. This
directory and the Kani harnesses next to the parser's code prove properties
of the scans for every input, which a measurement cannot do.

Two tools are used, because neither covers both halves:

| | Creusot (this crate) | Kani (`#[cfg(kani)]` in `src/`) |
| --- | --- | --- |
| Inputs covered | every length | every input up to a few bytes |
| Code checked | a copy of the production body, plus ghost code | the production code itself |
| What it can show | termination, running-time bounds, full functional specs | that two implementations agree, absence of panics |

A bounded check cannot show a running-time bound: below a fixed length,
every function is fast. So running-time claims come from Creusot, and Kani
checks that the memoized scans the parser actually calls agree with the
reference scans Creusot reasons about.

## What is proven

Creusot, for every input of every length:

- `mdx_jsx/braces.rs`: `skip_quoted`, `skip_line_comment`,
  `skip_block_comment` and `skip_braces` terminate, index only in bounds,
  never overflow, and run at most one loop iteration per byte between
  `start` and where they stop, the nested skips inside `skip_braces`
  included.
- `mdx_jsx/lex_memo.rs`: `LexMemo::backticks_end` returns the end of the
  first closer of the opener at `start`. This is a full spec,
  `first_closer` in `src/lex_memo.rs`, and it holds whatever earlier calls
  left in the memo. One call reads each byte after `start` at most once.
  `backtick_run_end` is proven too.

Kani, for every input of up to 6 bytes and every memo state two calls leave
behind:

- `LexMemo::quoted_end` returns what `braces::skip_quoted` returns.
- `LexMemo::backticks_end` returns what `braces::skip_backticks` returns.

## What is assumed

- **The copies match the code.** Each proven function is the production
  function plus ghost code (`ghost!`, `snapshot!`, `proof_assert!`,
  contracts, and a `steps` counter argument).
  `tools/scripts/check-formal-mirrors.mjs` strips the ghost code and
  compares what is left line by line with the production body. Creusot's
  own `#[erasure]` check would be stronger, but it rejects indexing
  (`bytes[i]`) in 0.15.
- **Cost model.** `steps` counts loop iterations. Every iteration does a
  constant amount of work besides the calls it counts. `memchr` is modelled
  as the plain loop it stands for.
- **Standard library.** `creusot-std` does not promise that `PartialEq`
  and `Ord` methods terminate, because a user's impl may loop. So
  `skip_braces` (which compares `Option<&u8>`s) and `backticks_end` (which
  calls `usize::max`) are proven without `#[check(terminates)]`: their own
  loops are still proven to terminate by their variants, and the std calls
  are trusted to return. Slices hold at most `isize::MAX` bytes (`fits`),
  which Rust guarantees and Creusot does not assume.

## What is not proven

The answer to "does every parser pass run in linear time?" is not yet. These
proofs cover single calls of single scans. A parse is linear only if, across
the whole document, each byte is scanned a bounded number of times. That
amortized argument depends on how callers retry after a miss and on every
memo the parser keeps (`code_runs`, `bracket_match`, the definition-term
run, the reference pre-pass chunk window, ...). It is still covered only by
the scaling tests. `LexMemo::brace_end` is checked by neither tool: CBMC
does not get through its hash map in reasonable time.

## Running

Creusot 0.15, from a clone of its repository (see its README for opam), or
through its flake:

```sh
nix build 'github:creusot-rs/creusot/v0.15.0#free' -o /tmp/creusot
PATH=/tmp/creusot/bin:$PATH cargo creusot   # in this directory
```

Kani 0.68, from the workspace root:

```sh
cargo install --locked kani-verifier --version 0.68.0 && cargo kani setup
cargo kani -p ox_content_parser -Z stubbing
```

`node tools/scripts/check-formal-mirrors.mjs` runs the mirror check.
`.github/workflows/formal.yml` runs all three on every change to the parser.

`why3find.proofs/` records which prover closed each goal, so a rerun
replays them instead of searching again. Commit it after changing a proof.
