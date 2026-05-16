# Changelog

## [2.7.0] - 2026-05-16

### Features

- support sidebar config (#86)
- support custom social links (#85)
- support git last updated (#84)
- render inline toc directive (#83)
- render toc outline in ssg theme (#82)
- add heading anchor ids (#80)

### Bug Fixes

- render inline raw html (#79)
- terminate html blocks on blank lines (#78)
- apply base to markdown paths (#77)
- pin deploy workflow actions (#76)
- pin benchmark workflow actions (#75)
- pin ci workflow actions (#74)
- harden publish workflow (#73)
- parse napi frontmatter with yaml (#72)
- report benchmark time and base speed (#71)
- harden renderer urls and workflows (#70)

## [2.6.0] - 2026-05-16

### Features

- use ox_jsdoc for docs generation (#69)
- add pull request benchmark comments (#64)

## [2.5.0] - 2026-05-07

### Performance

- speed up markdown render benchmark (#55)

### Documentation

- publish md4x benchmark results (#54)

## [2.4.0] - 2026-04-23

### Features

- unify ox content lsp and i18n tooling (#51)
- configurable markdown linting (#49)
- wasm (#46)

### Bug Fixes

- publish wasm package via npm (#48)
- publish wasm package via npm (#47)

## [2.3.0] - 2026-04-22

### Features

- wasm (#46)
- generated docs UX and scoped search (#44)
- code highlighting (#42)

### Bug Fixes

- text autosizing
- ci

## [2.2.0] - 2026-04-22

### Bug Fixes

- text autosizing

## [2.1.0] - 2026-04-22

## [2.0.0] - 2026-04-22

## [1.1.0] - 2026-04-21

### Features

- generated docs UX and scoped search (#44)
- code highlighting (#42)

## [1.0.0-alpha.0] - 2026-03-12

### Performance

- chunking common js/css assets (#36)

## [0.17.0] - 2026-03-12

## [0.16.0] - 2026-03-07

### Features

- new playground (#35)

## [0.15.0] - 2026-03-07

### Features

- perf tuning and more compat for mdast (#34)

## [0.14.0] - 2026-03-01

### Features

- i18n (#32)

## [0.13.0] - 2026-03-01

## [0.12.0] - 2026-02-23

### Features

- dev server (#31)

## [0.11.0] - 2026-02-22

## [0.10.0] - 2026-02-22

### Features

- block quote

## [0.9.0] - 2026-02-22

### Bug Fixes

- vue scoped css on og image

## [0.8.0] - 2026-02-22

### Features

- render og with public dir
- open graph viewer (#30)

## [0.7.0] - 2026-02-22

### Features

- render og with public dir
- open graph viewer (#30)

## [0.6.0] - 2026-02-21

### Bug Fixes

- twitter open graph meta (#29)

## [0.5.0] - 2026-02-21

### Bug Fixes

- open graph meta (#28)

## [0.4.0] - 2026-02-21

### Bug Fixes

- publishing

## [0.3.0-alpha.22] - 2026-02-21

### Bug Fixes

- publishing

## [0.3.0-alpha.21] - 2026-02-21

### Bug Fixes

- publishing

## [0.3.0-alpha.20] - 2026-02-21

### Bug Fixes

- publishing

## [0.3.0-alpha.19] - 2026-02-21

### Bug Fixes

- publishing

## [0.3.0-alpha.18] - 2026-02-21

### Bug Fixes

- publishing

## [0.3.0-alpha.17] - 2026-02-21

## [0.3.0-alpha.16] - 2026-02-21

### Features

- og feature (#26)

### Bug Fixes

- publishing
- ci
- ci

## [0.3.0-alpha.15] - 2026-02-20

### Bug Fixes

- ci
- ci

## [0.3.0-alpha.14] - 2026-02-20

### Features

- og feature (#26)

## [0.3.0-alpha.13] - 2026-02-19

## [0.3.0-alpha.12] - 2026-02-19

## [0.3.0-alpha.11] - 2026-02-19

## [0.3.0-alpha.10] - 2026-02-19

### Features

- native plugin (#23)
- theme api (#22)

### Bug Fixes

- publishing
- load ox-content.node binary name for napi-rs v3
- upgrade napi/napi-derive to v3 for index.d.ts generation
- remove optionalDependencies from source (added dynamically by napi pre-publish in CI)
- use --cross-compile instead of --zig for napi-rs v3
- publishing
- pass --no-sandbox to puppeteer for mermaid rendering in CI
- install chrome-headless-shell for mermaid-cli in CI
- type
- docs path

## [0.3.0-alpha.9] - 2026-02-10

### Bug Fixes

- publishing

## [0.3.0-alpha.8] - 2026-02-10

### Bug Fixes

- load ox-content.node binary name for napi-rs v3

## [0.3.0-alpha.7] - 2026-02-09

### Bug Fixes

- upgrade napi/napi-derive to v3 for index.d.ts generation
- remove optionalDependencies from source (added dynamically by napi pre-publish in CI)

## [0.3.0-alpha.6] - 2026-02-09

### Bug Fixes

- use --cross-compile instead of --zig for napi-rs v3

## [0.3.0-alpha.5] - 2026-02-09

### Features

- native plugin (#23)
- theme api (#22)

### Bug Fixes

- publishing
- pass --no-sandbox to puppeteer for mermaid rendering in CI
- install chrome-headless-shell for mermaid-cli in CI
- type
- docs path

## [0.3.0-alpha.4] - 2026-01-25

## [0.3.0-alpha.3] - 2026-01-25

### Features

- render content in markdown (#7)

## [0.3.0-alpha.2] - 2026-01-11

### Features

- use trusted publishing for crates.io

## [0.3.0-alpha.1] - 2026-01-11

## [0.3.0-alpha.0] - 2026-01-11

### Features

- search bar

### Bug Fixes

- ci
- ci

## [0.3.0] - 2026-01-08

## [0.2.0] - 2026-01-08

## [0.1.0] - 2026-01-08

### Features

- ssg and bench
- document generation (#3)
- unplugin
- docs
- docs
- docs
- other frameworks integration

### Bug Fixes

- fix map type in transform result
- resolve type errors in environment and transform
- add named exports for ESM compatibility
- `gen-source-docs` run script

### Documentation

- update README
