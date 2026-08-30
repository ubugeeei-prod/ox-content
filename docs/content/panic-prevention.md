# Panic Prevention

This is the first evidence ledger for [issue #774](https://github.com/ubugeeei-prod/ox-content/issues/774): malformed Markdown, config, paths, and plugin data must return errors or diagnostics instead of aborting the host process.

#774 is **not complete**. This page records the first mergeable slice: inventory, a CI gate, the most dangerous public-surface fixes, and the remaining exclusions.

Release binaries use `panic = "abort"`. A Rust panic in a published N-API artifact kills Node. `catch_unwind` only helps debug and test builds. The durable fix is to stop panicking on user input.

## Commands

```bash
# Inventory + allowlist gate (non-test Rust under crates/)
node scripts/check-panic-constructs.mjs

# Targeted regression tests for this slice
cargo test -p ox_content_parser --test input_panics
cargo test -p ox_content_ssg --lib paths -- --nocapture
cargo test -p ox_content_transform --lib hostile_user_content
cargo test -p ox_content_renderer --lib svelte_public_codegen
cargo test -p ox_content_napi hostile_markdown
```

CI runs the gate as the `Panic constructs` job in `.github/workflows/ci.yml`. `vp run check:panic-constructs` and `vp run workspace:check` run the same script.

## Audited subsystems (this slice)

| Subsystem                       | Crate                  | Outcome                                                                                                                                                                                                                          |
| ------------------------------- | ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Markdown parse                  | `ox_content_parser`    | Public `parse` returns `ParseResult`. Sub-parsers inherit nesting depth so deeply nested quotes return `NestingTooDeep` instead of growing without bound. Delimiter and list helpers no longer `expect` on recoverable mismatch. |
| HTML render / framework codegen | `ox_content_renderer`  | String writes no longer `expect`. Svelte public codegen no longer hits `unreachable!`.                                                                                                                                           |
| Transform / frontmatter         | `ox_content_transform` | Malformed YAML stays empty frontmatter. Hostile Markdown returns `errors` instead of aborting. Compile-time YouTube regex `expect`s remain allowlisted.                                                                          |
| SSG routes / entry links        | `ox_content_ssg`       | Path suffix stripping is byte-safe. Multibyte paths such as `😀` no longer slice mid-character.                                                                                                                                  |
| N-API public parse / transform  | `ox_content_napi`      | Cache misses return `napi::Error`. Parse/transform wrap unexpected panics into the `errors` array in unwind builds.                                                                                                              |

Test-only `unwrap` / `expect` / `panic!` are out of scope.

## Fixes in this slice

- **SSG paths**: `strip_markdown_extension` compared the last N bytes with a `&str` slice. A 4-byte emoji path (`😀`) panicked at a non-character boundary before the comparison ran. The helper now compares ASCII suffixes on bytes.
- **SSG entry links**: `.md` stripping uses the same byte-safe suffix check.
- **Parser lists / emphasis**: local `expect`s after initialization or retain are now `get_or_insert_with` / `if let`.
- **Parser nesting**: every sub-source parser — block quote, list item, footnote body, JSX children — is built one level deeper than its parent, so `max_nesting_depth` bounds the recursion of a parse however the constructs are combined. It defaults to 100 even without the GFM profile, because `0` (unlimited) lets a deeply nested document overflow the stack, and a stack overflow aborts rather than unwinds.
- **Renderer / SWAR scans**: 8-byte `try_into().unwrap()` copies into a stack array after a length check.
- **N-API cache**: a missing transformed file is a `Result` error, not `expect`.
- **N-API FFI**: `parse`, `parse_and_render`, and `transform` recover from unexpected panics in unwind builds.

## CI gate and allowlist

`scripts/check-panic-constructs.mjs` walks `crates/**/*.rs`, skips `tests/`, `benches/`, `examples/`, `tests.rs`, and `#[cfg(test)]` modules, then counts:

`unwrap`, `unwrap_err`, `unwrap_unchecked`, `expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`

Counts are compared to `config/panic-allowlist.json`. New hits fail. Lower actual counts also fail until the allowlist is reduced. This is not a workspace-wide `allow(clippy::unwrap_used)`.

The five focus crates also `deny` `clippy::unwrap_used`, `expect_used`, `panic`, `todo`, and `unimplemented` for non-test builds. The only reviewed exception in those crates is the compile-time YouTube regex in `ox_content_transform`.

## Remaining work (follow-up PRs)

- Finish the rest of the workspace: `ox_content_docs`, `ox_content_lsp`, `ox_content_i18n`, `ox_content_search`, `ox_content_highlight`, `ox_content_wasm`, Vite bindings, and editor crates.
- Add bounded fuzz / property lanes in CI (the existing `fuzz/` targets still need nightly and are not a required CI job).
- Decide whether published artifacts can use unwind at FFI boundaries instead of `panic = "abort"`.
- Keep shrinking `config/panic-allowlist.json` as each remaining site is proven or rewritten.

Do not close #774 until those items are done.
