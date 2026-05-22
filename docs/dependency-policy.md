# Dependency Policy

CI gates dependency advisories and licenses for both Rust and npm surfaces.

## Advisories

- Cargo advisories run through `cargo audit`.
- npm advisories run through `scripts/check-npm-advisories.mjs`.
- The npm gate fails on new `high` or `critical` advisories unless an item is explicitly listed in `config/dependency-policy.json`.

Allowed advisories must include a reason and an expiry date. Remove the entry when the dependency is upgraded or the advisory no longer applies.

## Licenses

- Cargo license policy lives in `deny.toml` and is checked by `cargo deny check licenses`.
- npm license policy lives in `config/dependency-policy.json` and is checked by `scripts/check-npm-licenses.mjs`.

Prefer permissive licenses such as MIT, Apache-2.0, BSD, ISC, Unicode-3.0, Zlib, CC0, MPL-2.0, and Unlicense. Exceptions must name the package, license, and reason so maintainers can audit them during dependency refreshes.
