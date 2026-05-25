# Fuzzing

`cargo-fuzz` targets for the Markdown parser and HTML renderer. This crate is
intentionally a separate workspace so `libfuzzer-sys` and the nightly
sanitizer requirements stay out of the main build.

## Prerequisites

```bash
cargo install cargo-fuzz
rustup toolchain install nightly  # libfuzzer needs nightly
```

## Targets

| target            | what it checks                                                                |
| ----------------- | ----------------------------------------------------------------------------- |
| `parse`           | parser with default options never panics on any UTF-8 input                   |
| `parse_gfm`       | parser with GFM extensions enabled never panics                               |
| `parse_render`    | parser + renderer pipeline never panics; options derived from input prefix    |
| `render_sanitize` | sanitize=true output never emits `href="javascript:` / `src="javascript:`     |

## Running

From the `fuzz/` directory:

```bash
# Quick smoke run for a few seconds.
cargo +nightly fuzz run parse -- -runs=10000

# Longer continuous run.
cargo +nightly fuzz run parse_render

# Reproduce a crash from `artifacts/`.
cargo +nightly fuzz run parse_render artifacts/parse_render/crash-...
```

Seed inputs live under `corpus/<target>/` and are loaded automatically by
libFuzzer.
