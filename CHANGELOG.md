# Changelog

## [2.18.0] - 2026-05-25

### Bug Fixes

- remove panic-prone runtime paths (#171)
- improve docs hero, search, and source links (#169)
- render docs assets in CI

### Performance

- arena strings, dispatch cache, fewer heading allocs (#172)

### Documentation

- separate user guide and advanced docs
- expand performance documentation (#167)
- update architecture overview (#166)

## [2.17.0] - 2026-05-25

## [2.16.0] - 2026-05-25

### Performance

- byte-level fast paths and zero-copy hot spots (#164)

### Documentation

- add community credits (@kazupon)

## [2.15.0] - 2026-05-25

### Features

- add allocation and timing profiling mode (#163)
- expose and render API members (#160)

## [2.14.0] - 2026-05-25

### Features

- filter internal declarations (#156)
- resolve entrypoint export graph (#158)
- extract file-level module jsdoc (#157)
- extract plain variable declarations (#155)
- add component checker diagnostics
- add builtin open graph embeds (#101)
- add builtin github embeds (#99)
- support mdx content files (#98)
- migration path (#39)
- mdast js plugin (#40)
- add Intl localization helpers (#87)
- add runtime path helpers (#88)
- support sidebar config (#86)
- support custom social links (#85)
- support git last updated (#84)
- render inline toc directive (#83)
- render toc outline in ssg theme (#82)
- add heading anchor ids (#80)
- use ox_jsdoc for docs generation (#69)
- add pull request benchmark comments (#64)
- unify ox content lsp and i18n tooling (#51)
- configurable markdown linting (#49)
- wasm (#46)
- generated docs UX and scoped search (#44)
- code highlighting (#42)

### Bug Fixes

- align vite-plus-core catalog with vite-plus (#146)
- protect public export surface
- escape bare page titles
- render list item fenced code as blocks
- add spacing below expanded docs entries (#134)
- harden embed inputs (#89)
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
- publish wasm package via npm (#48)
- publish wasm package via npm (#47)
- text autosizing
- ci

### Performance

- batch-parse JSDoc comments in extractor (#111)
- reduce search query allocations (#97)
- speed up markdown render benchmark (#55)

### Refactoring

- centralize metadata in Rust
- move i18n project checks into napi (#109)
- move bare ssg html into rust (#110)
- type search module options (#108)
- write search index in rust (#107)
- build search index in rust (#106)
- move docs and ssg helpers to rust (#105)
- remove mod.rs module roots (#104)
- move docs nav generation to Rust (#96)
- move search runtime generation to Rust (#95)
- move SSG routing helpers to Rust (#94)
- move SSG asset externalization to Rust (#93)
- move docs normalization to Rust (#92)
- move i18n runtime generation to Rust (#90)

### Documentation

- add security policy (#126)
- add contributing guide (#127)
- publish md4x benchmark results (#54)

## [2.13.0] - 2026-05-24

### Bug Fixes

- align vite-plus-core catalog with vite-plus (#146)

## [2.12.0] - 2026-05-24

### Features

- add component checker diagnostics

### Bug Fixes

- protect public export surface
- escape bare page titles
- render list item fenced code as blocks
- add spacing below expanded docs entries (#134)

### Performance

- batch-parse JSDoc comments in extractor (#111)

### Refactoring

- centralize metadata in Rust
- move i18n project checks into napi (#109)
- move bare ssg html into rust (#110)
- type search module options (#108)
- write search index in rust (#107)
- build search index in rust (#106)
- move docs and ssg helpers to rust (#105)
- remove mod.rs module roots (#104)

### Documentation

- add security policy (#126)
- add contributing guide (#127)

## [2.11.0] - 2026-05-16

### Features

- add builtin open graph embeds (#101)
- add builtin github embeds (#99)
- support mdx content files (#98)

## [2.10.0] - 2026-05-16

### Features

- migration path (#39)
- mdast js plugin (#40)

## [2.9.0] - 2026-05-16

### Performance

- reduce search query allocations (#97)

### Refactoring

- move docs nav generation to Rust (#96)
- move search runtime generation to Rust (#95)
- move SSG routing helpers to Rust (#94)
- move SSG asset externalization to Rust (#93)
- move docs normalization to Rust (#92)
- move i18n runtime generation to Rust (#90)

## [2.8.0] - 2026-05-16

### Features

- add Intl localization helpers (#87)
- add runtime path helpers (#88)

### Bug Fixes

- harden embed inputs (#89)

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
