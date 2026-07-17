# Spec fixtures

- `commonmark-0.31.2-spec.txt` is the unmodified CommonMark specification
  `spec.txt` (version 0.31.2) by John MacFarlane, vendored from
  <https://github.com/commonmark/commonmark-spec/blob/0.31.2/spec.txt> and
  licensed under [CC-BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
  It is used here only as test data for the conformance suite in
  `tests/spec_commonmark.rs`.
- `commonmark-known-failures.txt` tracks the spec examples ox-content does
  not render per spec yet. The conformance test fails when an entry starts
  passing (remove the line) or when a conforming example regresses.
  Regenerate with:

  ```sh
  UPDATE_SPEC_BASELINE=1 cargo test -p ox_content_renderer --test spec_commonmark
  ```
