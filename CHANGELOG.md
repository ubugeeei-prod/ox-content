# Changelog

## [2.79.0] - 2026-07-17

## [2.78.1] - 2026-07-17

### Bug Fixes

- align table rendering with GFM (#482)
- configure npm release authentication (#481)

## [2.78.0] - 2026-07-15

### Features

- fetch static tweet content (#479)

### Bug Fixes

- forward autolinks option (#478)
- preserve escaped table pipes (#476)
- derive code block gradient color (#477)
- make editor publishing idempotent
- authenticate npm release jobs

## [2.77.0] - 2026-07-13

### Bug Fixes

- repair API docs and playground assets
- support older glibc and musl napi builds (#460)
- harden publish workflow

## [2.76.0] - 2026-07-01

### Features

- add markdown collection queries (#450)

### Bug Fixes

- export declared wrapper functions (#458)
- unwrap default export of ESM-only rehype plugins in CJS build (#452)
- publish mdast before dependent crates
- handle IME search and crate publish order (#447)

### Documentation

- update collection API reference
- document release operations

## [2.75.1] - 2026-06-24

### Bug Fixes

- unwrap default export of ESM-only rehype plugins in CJS build (#452)

### Documentation

- update collection API reference
- document release operations

## [2.75.0] - 2026-06-23

### Features

- add markdown collection queries (#450)

### Bug Fixes

- handle IME search and crate publish order (#447)
- prevent last mobile menu item from being hidden behind the footer (#440)
- restore main workflow checks (#446)

## [2.74.0] - 2026-06-23

### Features

- add framework markdown render utilities
- wire textlint editor integration (#426)
- support jsdoc throws tags (#374)
- persist sticky sidebar state (#371)
- support flattened single entry roots (#361)
- opt-in incremental markdown parsing/rendering (#357)
- render jsdoc member default values (#356)
- add document highlights for matching link targets (#355)
- add smart selection ranges (expand selection) (#350)
- add document links for Markdown links and images (#344)
- add folding ranges for headings, code blocks, and frontmatter (#343)
- support NAPI docs options (#339)
- add renderGeneratedBy option (#335)
- add JS/TS docs-generator profiling mode (#309)
- add sort, sortEntryPoints, and kindSortOrder organization options (#307)
- add groupOrder option to control typedoc section and nav group order (#299)
- add renderStats option to omit generated markdown stats summaries (#298)
- add Markdown display formats (#284)
- add vitest docs test harness (#271)
- add opt-in type parameter docs (#272)
- add opt-in content transforms (#265)
- add pure markdown render mode via renderStyle option (#261)
- package-manager install tabs with opt-in synced groups (#257)
- complete typedoc path strategy support (#214)
- autolink bare URLs in text (#205)
- scaffold crate (#196)
- preview HMR push channel (#192)
- on-save LSP sidecar with opt-in command override (#197)
- component name + attribute completion via registry (#195)
- new crate, LSP diagnostics, CLI (#193)
- asset path completion inside link/image openers (#194)
- support clean URLs in generated Markdown links (#187)

### Bug Fixes

- prevent last mobile menu item from being hidden behind the footer (#440)
- restore main workflow checks (#446)
- generate valid docs nav TypeScript (#436)
- use crates.io environment for publishing (#373)
- lower linux x64 binding glibc baseline (#369)
- detect helper-based cargo publish targets (#362)
- preserve napi declaration docs (#359)
- allow media embeds through sanitizer (#342)
- render nested HTML member formats (#338)
- avoid duplicate property returns (#337)
- strip JSDoc from type alias signatures (#336)
- omit empty type parameter descriptions (#334)
- resolve intersection callable aliases (#333)
- merge destructured param docs (#332)
- suppress property returns sections (#331)
- render member type parameters (#330)
- expand object literal params (#329)
- avoid escaping return union pipes (#328)
- preserve function type alias metadata (#326)
- preserve function-valued property types (#325)
- render TypeScript index signatures (#324)
- render class method details (#323)
- render return type literal members (#322)
- collapse multiline type params (#321)
- do not double-wrap mixed markdown @example bodies in a code fence (#320)
- use entry source path for typedoc module index source link (#308)
- never link TypeScript primitive types in annotations (#306)
- link known symbols inside rendered type annotations (#305)
- drop redundant Kind column from named member tables (#304)
- sort class/interface/type members alphabetically to match typedoc (#303)
- sort and dedupe typedoc nav leaf entries to match markdown order (#302)
- bring html render style to feature parity with markdown (#300)
- render all overload call signatures on typedoc symbol pages (#297)
- render @since and @version as a Since section in markdown output (#296)
- include declaration kind in typedoc symbol page H1 titles (#294)
- render @experimental and @deprecated as GitHub alerts in markdown output (#293)
- render module-level examples (#292)
- preserve typedoc module names (#283)
- format module index references as typedoc-style heading entries (#282)
- render typedoc module index members as compact tables instead of bullet lists (#281)
- emit one canonical typedoc page per symbol for cross-entrypoint re-exports (#280)
- render pure markdown sections as sequential headings instead of bold paragraphs (#275)
- extract module description without @module and across split header comments (#274)
- drop source links for external dependency symbols (#270)
- carry module-level @module description through to generated output (#268)
- deploy docs from void root
- restore Bun.markdown row in PR benchmark (#258)
- remove needless raw string hashes in tabs tests
- support typedoc markdown paths (#209)
- render JSDoc inline links (#204)
- document local entrypoint exports (#199)
- extract external re-export docs (#198)

### Performance

- append Markdown table cells directly instead of per-cell Strings (#319)
- extract docs during the export-graph walk to avoid a second parse (#318)
- reduce TypeDoc render allocations (symbol map + list rows) (#317)
- skip raw JSDoc text and param formatting on normalize paths (#316)
- reuse the OXC arena allocator across files (#314)
- borrow instead of allocate in doc-text/link processing (#315)
- reduce markdown renderer allocations (#295)
- fast-path block dispatch (#290)
- fast-path simple list items (#289)
- optimize Rust hot paths and release profiles (#287)
- optimize html block parsing (#286)
- optimize docs markdown rendering (#285)
- debug-build NAPI smoke and cache rendering browsers (#263)
- lazily bucket members and drop format! in pure markdown renderer (#262)
- search runtime (#241)
- borrow frontmatter content and move the autolink patterns (#235)
- resolve spellcheck issue lines via binary search (#236)
- gate text autolinking on the cached autolink_index (#234)
- drop redundant allocations in slugify and the YouTube embed (#233)
- cut per-symbol allocations on the generation path (#232)
- tighten leaf/list/fenced block scans (#231)
- SIMD-accelerate inline scanning with memchr (#230)
- hoist per-page constant work out of the page loop (#227)
- memoize per-doc scopes and the prefix-scan vocabulary (#228)
- build the autolink first-byte index once per render (#225)
- cache sort keys and bucket members lazily (#226)
- skip redundant block dispatch on a paragraph's first line (#222)
- SIMD-accelerate the static embed transforms (#224)
- port the tabs embed transform to Rust (#221)
- port the YouTube embed transform to Rust (#220)
- skip no-op rehype round-trips and redundant per-page work (#218)
- skip non-URL text in autolink scan via memchr (#217)
- optimize release profile (#202)
- add Allocator::for_source_len, use it across LSP + NAPI (#190)
- fast-path text in inline dispatch, pre-size heading scratch (#188)
- reuse parsed list lines (#184)
- scan safe urls in chunks (#183)
- write numeric attrs without strings (#182)
- avoid unused link url allocations (#181)
- write duplicate toc id suffixes in place (#180)
- avoid duplicate toc slug clones (#179)
- avoid inline toc entry clones (#178)
- avoid temporary table row allocations (#177)
- write heading id directly to output, skip callout alloc (#173) (#174)

### Refactoring

- move framework codegen behind feature flag (#438)
- split napi logic into core crates (#435)
- split final oversized rust files
- split wasm modules
- split link checker modules
- split search query tests
- split remaining napi modules (#421)
- split parser modules
- split i18n modules
- split napi transform helpers
- split napi lint sanitize modules
- split profile cli modules
- split profiler modules
- split renderer tests
- split ssg html rendering
- split docs export graph
- split docs markdown pure renderer
- split docs markdown html renderer
- split docs markdown pages
- split docs extractor visitor
- split napi lint helpers
- split napi emoji lookup
- split napi mdast raw serialization
- split napi transform bindings
- split napi docs bindings
- split napi pm helpers
- split napi feature helpers
- split docs graph export helpers
- split docs graph entrypoint helpers
- split docs graph resolver
- split docs pure member rendering
- split docs html member rendering
- split docs markdown linking helpers
- split docs extractor driver helpers
- split docs markdown metadata helpers
- split docs markdown core helpers
- split docs markdown renderers
- split docs extractor and markdown tests
- split docs data nav normalize modules
- split docs graph tests
- split docs public models
- split ssg route and asset helpers (#386)
- split napi feature helpers (#385)
- split ssg html modules (#384)
- split docs crate helpers (#383)
- split extractor tag helpers
- split markdown ordering helpers (#381)
- split docs Rust modules (#379)
- split GitHub embed plugin (#378)
- use compact strings for small state (#377)
- prefer fx hash collections (#376)
- add defaults for docs fixtures (#375)
- split large binding and markdown test modules (#358)
- split html renderer modules (#266)
- split long implementations (#203)
- move VitePress frontmatter normalization to Rust (#186)
- move docs generation output into rust (#185)

### Documentation

- add kazupon credits summary (#367)
- add builtin examples and framework tests (#360)
- document optimization hot paths (#291)
- expand built-in feature and Void deploy guides (#269)
- format generated API reference
- refresh generated API reference
- add an MDX & Components guide (#240)
- add JSDoc API-docs and i18n guides (#239)
- document dark mode, embed slots, social icons, custom CSS (#238)
- update benchmarks
- add editor extension roadmap (#189)

## [2.73.0] - 2026-06-22

### Features

- add framework markdown render utilities
- wire textlint editor integration (#426)

### Bug Fixes

- generate valid docs nav TypeScript (#436)

### Refactoring

- move framework codegen behind feature flag (#438)
- split napi logic into core crates (#435)
- split final oversized rust files
- split wasm modules
- split link checker modules
- split search query tests
- split remaining napi modules (#421)
- split parser modules
- split i18n modules
- split napi transform helpers
- split napi lint sanitize modules
- split profile cli modules
- split profiler modules
- split renderer tests
- split ssg html rendering
- split docs export graph
- split docs markdown pure renderer
- split docs markdown html renderer
- split docs markdown pages
- split docs extractor visitor
- split napi lint helpers
- split napi emoji lookup
- split napi mdast raw serialization
- split napi transform bindings
- split napi docs bindings
- split napi pm helpers
- split napi feature helpers
- split docs graph export helpers
- split docs graph entrypoint helpers
- split docs graph resolver
- split docs pure member rendering
- split docs html member rendering
- split docs markdown linking helpers
- split docs extractor driver helpers
- split docs markdown metadata helpers
- split docs markdown core helpers
- split docs markdown renderers
- split docs extractor and markdown tests
- split docs data nav normalize modules
- split docs graph tests
- split docs public models
- split ssg route and asset helpers (#386)
- split napi feature helpers (#385)
- split ssg html modules (#384)
- split docs crate helpers (#383)
- split extractor tag helpers
- split markdown ordering helpers (#381)
- split docs Rust modules (#379)
- split GitHub embed plugin (#378)

## [2.72.0] - 2026-06-21

### Features

- add framework markdown render utilities
- wire textlint editor integration (#426)
- support jsdoc throws tags (#374)
- persist sticky sidebar state (#371)
- support flattened single entry roots (#361)
- opt-in incremental markdown parsing/rendering (#357)
- render jsdoc member default values (#356)
- add document highlights for matching link targets (#355)
- add smart selection ranges (expand selection) (#350)

### Bug Fixes

- generate valid docs nav TypeScript (#436)
- use crates.io environment for publishing (#373)
- lower linux x64 binding glibc baseline (#369)
- detect helper-based cargo publish targets (#362)
- preserve napi declaration docs (#359)

### Refactoring

- move framework codegen behind feature flag (#438)
- split napi logic into core crates (#435)
- split final oversized rust files
- split wasm modules
- split link checker modules
- split search query tests
- split remaining napi modules (#421)
- split parser modules
- split i18n modules
- split napi transform helpers
- split napi lint sanitize modules
- split profile cli modules
- split profiler modules
- split renderer tests
- split ssg html rendering
- split docs export graph
- split docs markdown pure renderer
- split docs markdown html renderer
- split docs markdown pages
- split docs extractor visitor
- split napi lint helpers
- split napi emoji lookup
- split napi mdast raw serialization
- split napi transform bindings
- split napi docs bindings
- split napi pm helpers
- split napi feature helpers
- split docs graph export helpers
- split docs graph entrypoint helpers
- split docs graph resolver
- split docs pure member rendering
- split docs html member rendering
- split docs markdown linking helpers
- split docs extractor driver helpers
- split docs markdown metadata helpers
- split docs markdown core helpers
- split docs markdown renderers
- split docs extractor and markdown tests
- split docs data nav normalize modules
- split docs graph tests
- split docs public models
- split ssg route and asset helpers (#386)
- split napi feature helpers (#385)
- split ssg html modules (#384)
- split docs crate helpers (#383)
- split extractor tag helpers
- split markdown ordering helpers (#381)
- split docs Rust modules (#379)
- split GitHub embed plugin (#378)
- use compact strings for small state (#377)
- prefer fx hash collections (#376)
- add defaults for docs fixtures (#375)
- split large binding and markdown test modules (#358)

### Documentation

- add kazupon credits summary (#367)
- add builtin examples and framework tests (#360)

## [2.71.0] - 2026-06-21

### Features

- add framework markdown render utilities
- wire textlint editor integration (#426)

### Bug Fixes

- generate valid docs nav TypeScript (#436)

### Refactoring

- split napi logic into core crates (#435)
- split final oversized rust files
- split wasm modules
- split link checker modules
- split search query tests
- split remaining napi modules (#421)
- split parser modules
- split i18n modules
- split napi transform helpers
- split napi lint sanitize modules
- split profile cli modules
- split profiler modules
- split renderer tests
- split ssg html rendering
- split docs export graph
- split docs markdown pure renderer
- split docs markdown html renderer
- split docs markdown pages
- split docs extractor visitor
- split napi lint helpers
- split napi emoji lookup
- split napi mdast raw serialization
- split napi transform bindings
- split napi docs bindings
- split napi pm helpers
- split napi feature helpers
- split docs graph export helpers
- split docs graph entrypoint helpers
- split docs graph resolver
- split docs pure member rendering
- split docs html member rendering
- split docs markdown linking helpers
- split docs extractor driver helpers
- split docs markdown metadata helpers
- split docs markdown core helpers
- split docs markdown renderers
- split docs extractor and markdown tests
- split docs data nav normalize modules
- split docs graph tests
- split docs public models
- split ssg route and asset helpers (#386)
- split napi feature helpers (#385)
- split ssg html modules (#384)
- split docs crate helpers (#383)
- split extractor tag helpers
- split markdown ordering helpers (#381)
- split docs Rust modules (#379)
- split GitHub embed plugin (#378)

## [2.70.0] - 2026-06-11

### Features

- support jsdoc throws tags (#374)

### Refactoring

- use compact strings for small state (#377)
- prefer fx hash collections (#376)
- add defaults for docs fixtures (#375)

## [2.69.0] - 2026-06-11

### Features

- support jsdoc throws tags (#374)
- persist sticky sidebar state (#371)

### Bug Fixes

- use crates.io environment for publishing (#373)

## [2.68.0] - 2026-06-10

### Bug Fixes

- use crates.io environment for publishing (#373)

## [2.67.0] - 2026-06-10

### Features

- persist sticky sidebar state (#371)
- support flattened single entry roots (#361)

### Bug Fixes

- lower linux x64 binding glibc baseline (#369)
- detect helper-based cargo publish targets (#362)
- preserve napi declaration docs (#359)

### Refactoring

- split large binding and markdown test modules (#358)

### Documentation

- add kazupon credits summary (#367)
- add builtin examples and framework tests (#360)

## [2.66.0] - 2026-06-10

### Bug Fixes

- lower linux x64 binding glibc baseline (#369)

### Documentation

- add kazupon credits summary (#367)

## [2.65.0] - 2026-06-09

### Features

- support flattened single entry roots (#361)

### Bug Fixes

- detect helper-based cargo publish targets (#362)
- preserve napi declaration docs (#359)

### Refactoring

- split large binding and markdown test modules (#358)

### Documentation

- add builtin examples and framework tests (#360)

## [2.64.0] - 2026-06-08

### Features

- opt-in incremental markdown parsing/rendering (#357)

## [2.63.0] - 2026-06-08

### Features

- render jsdoc member default values (#356)
- add document highlights for matching link targets (#355)
- add smart selection ranges (expand selection) (#350)
- add document links for Markdown links and images (#344)
- add folding ranges for headings, code blocks, and frontmatter (#343)

### Bug Fixes

- allow media embeds through sanitizer (#342)

## [2.62.0] - 2026-06-07

### Features

- add document links for Markdown links and images (#344)
- add folding ranges for headings, code blocks, and frontmatter (#343)
- support NAPI docs options (#339)
- add renderGeneratedBy option (#335)
- add JS/TS docs-generator profiling mode (#309)
- add sort, sortEntryPoints, and kindSortOrder organization options (#307)

### Bug Fixes

- allow media embeds through sanitizer (#342)
- render nested HTML member formats (#338)
- avoid duplicate property returns (#337)
- strip JSDoc from type alias signatures (#336)
- omit empty type parameter descriptions (#334)
- resolve intersection callable aliases (#333)
- merge destructured param docs (#332)
- suppress property returns sections (#331)
- render member type parameters (#330)
- expand object literal params (#329)
- avoid escaping return union pipes (#328)
- preserve function type alias metadata (#326)
- preserve function-valued property types (#325)
- render TypeScript index signatures (#324)
- render class method details (#323)
- render return type literal members (#322)
- collapse multiline type params (#321)
- do not double-wrap mixed markdown @example bodies in a code fence (#320)
- use entry source path for typedoc module index source link (#308)

### Performance

- append Markdown table cells directly instead of per-cell Strings (#319)
- extract docs during the export-graph walk to avoid a second parse (#318)
- reduce TypeDoc render allocations (symbol map + list rows) (#317)
- skip raw JSDoc text and param formatting on normalize paths (#316)
- reuse the OXC arena allocator across files (#314)
- borrow instead of allocate in doc-text/link processing (#315)

## [2.61.0] - 2026-06-06

### Features

- support NAPI docs options (#339)

### Bug Fixes

- render nested HTML member formats (#338)

## [2.60.0] - 2026-06-06

## [2.59.0] - 2026-06-06

### Features

- add renderGeneratedBy option (#335)

### Bug Fixes

- avoid duplicate property returns (#337)
- strip JSDoc from type alias signatures (#336)
- omit empty type parameter descriptions (#334)

## [2.58.0] - 2026-06-06

### Features

- add renderGeneratedBy option (#335)

### Bug Fixes

- avoid duplicate property returns (#337)
- strip JSDoc from type alias signatures (#336)
- omit empty type parameter descriptions (#334)

## [2.57.0] - 2026-06-05

### Bug Fixes

- resolve intersection callable aliases (#333)
- merge destructured param docs (#332)
- suppress property returns sections (#331)

## [2.56.0] - 2026-06-05

### Bug Fixes

- render member type parameters (#330)
- expand object literal params (#329)
- avoid escaping return union pipes (#328)

## [2.55.0] - 2026-06-05

### Bug Fixes

- preserve function type alias metadata (#326)
- preserve function-valued property types (#325)
- render TypeScript index signatures (#324)
- render class method details (#323)
- render return type literal members (#322)
- collapse multiline type params (#321)
- do not double-wrap mixed markdown @example bodies in a code fence (#320)

## [2.54.0] - 2026-06-04

### Features

- add JS/TS docs-generator profiling mode (#309)

### Performance

- append Markdown table cells directly instead of per-cell Strings (#319)
- extract docs during the export-graph walk to avoid a second parse (#318)
- reduce TypeDoc render allocations (symbol map + list rows) (#317)
- skip raw JSDoc text and param formatting on normalize paths (#316)
- reuse the OXC arena allocator across files (#314)
- borrow instead of allocate in doc-text/link processing (#315)

## [2.53.0] - 2026-06-04

## [2.52.0] - 2026-06-04

## [2.51.0] - 2026-06-04

### Features

- add sort, sortEntryPoints, and kindSortOrder organization options (#307)

### Bug Fixes

- use entry source path for typedoc module index source link (#308)

## [2.50.0] - 2026-06-03

### Features

- add groupOrder option to control typedoc section and nav group order (#299)

### Bug Fixes

- never link TypeScript primitive types in annotations (#306)
- link known symbols inside rendered type annotations (#305)
- drop redundant Kind column from named member tables (#304)
- sort class/interface/type members alphabetically to match typedoc (#303)
- sort and dedupe typedoc nav leaf entries to match markdown order (#302)
- bring html render style to feature parity with markdown (#300)

## [2.49.0] - 2026-06-03

### Bug Fixes

- link known symbols inside rendered type annotations (#305)

## [2.48.0] - 2026-06-03

### Bug Fixes

- drop redundant Kind column from named member tables (#304)

## [2.47.0] - 2026-06-03

### Bug Fixes

- sort class/interface/type members alphabetically to match typedoc (#303)
- sort and dedupe typedoc nav leaf entries to match markdown order (#302)

## [2.46.0] - 2026-06-03

### Features

- add groupOrder option to control typedoc section and nav group order (#299)
- add renderStats option to omit generated markdown stats summaries (#298)

### Bug Fixes

- bring html render style to feature parity with markdown (#300)
- render all overload call signatures on typedoc symbol pages (#297)

## [2.45.0] - 2026-06-03

### Features

- add renderStats option to omit generated markdown stats summaries (#298)

### Bug Fixes

- render all overload call signatures on typedoc symbol pages (#297)

## [2.44.0] - 2026-06-02

### Bug Fixes

- render @since and @version as a Since section in markdown output (#296)

### Performance

- reduce markdown renderer allocations (#295)

## [2.43.0] - 2026-06-02

### Bug Fixes

- include declaration kind in typedoc symbol page H1 titles (#294)
- render @experimental and @deprecated as GitHub alerts in markdown output (#293)

## [2.42.0] - 2026-06-02

### Features

- add Markdown display formats (#284)

### Bug Fixes

- render module-level examples (#292)
- preserve typedoc module names (#283)

### Performance

- fast-path block dispatch (#290)
- fast-path simple list items (#289)
- optimize Rust hot paths and release profiles (#287)
- optimize html block parsing (#286)
- optimize docs markdown rendering (#285)

### Documentation

- document optimization hot paths (#291)

## [2.41.0] - 2026-06-01

### Performance

- optimize Rust hot paths and release profiles (#287)
- optimize html block parsing (#286)
- optimize docs markdown rendering (#285)

## [2.40.0] - 2026-06-01

### Features

- add Markdown display formats (#284)

### Bug Fixes

- preserve typedoc module names (#283)
- format module index references as typedoc-style heading entries (#282)
- render typedoc module index members as compact tables instead of bullet lists (#281)
- emit one canonical typedoc page per symbol for cross-entrypoint re-exports (#280)

## [2.39.0] - 2026-06-01

### Bug Fixes

- format module index references as typedoc-style heading entries (#282)

## [2.38.0] - 2026-06-01

### Features

- add vitest docs test harness (#271)
- add opt-in type parameter docs (#272)
- add opt-in content transforms (#265)
- add pure markdown render mode via renderStyle option (#261)
- package-manager install tabs with opt-in synced groups (#257)

### Bug Fixes

- render typedoc module index members as compact tables instead of bullet lists (#281)
- emit one canonical typedoc page per symbol for cross-entrypoint re-exports (#280)
- render pure markdown sections as sequential headings instead of bold paragraphs (#275)
- extract module description without @module and across split header comments (#274)
- drop source links for external dependency symbols (#270)
- carry module-level @module description through to generated output (#268)
- deploy docs from void root
- restore Bun.markdown row in PR benchmark (#258)
- remove needless raw string hashes in tabs tests

### Performance

- debug-build NAPI smoke and cache rendering browsers (#263)
- lazily bucket members and drop format! in pure markdown renderer (#262)
- search runtime (#241)
- borrow frontmatter content and move the autolink patterns (#235)
- resolve spellcheck issue lines via binary search (#236)
- gate text autolinking on the cached autolink_index (#234)
- drop redundant allocations in slugify and the YouTube embed (#233)
- cut per-symbol allocations on the generation path (#232)
- tighten leaf/list/fenced block scans (#231)
- SIMD-accelerate inline scanning with memchr (#230)
- hoist per-page constant work out of the page loop (#227)
- memoize per-doc scopes and the prefix-scan vocabulary (#228)
- build the autolink first-byte index once per render (#225)
- cache sort keys and bucket members lazily (#226)
- skip redundant block dispatch on a paragraph's first line (#222)
- SIMD-accelerate the static embed transforms (#224)

### Refactoring

- split html renderer modules (#266)

### Documentation

- expand built-in feature and Void deploy guides (#269)
- format generated API reference
- refresh generated API reference
- add an MDX & Components guide (#240)
- add JSDoc API-docs and i18n guides (#239)
- document dark mode, embed slots, social icons, custom CSS (#238)

## [2.37.0] - 2026-06-01

### Features

- add vitest docs test harness (#271)
- add opt-in type parameter docs (#272)
- add opt-in content transforms (#265)
- add pure markdown render mode via renderStyle option (#261)
- package-manager install tabs with opt-in synced groups (#257)

### Bug Fixes

- render typedoc module index members as compact tables instead of bullet lists (#281)
- emit one canonical typedoc page per symbol for cross-entrypoint re-exports (#280)
- render pure markdown sections as sequential headings instead of bold paragraphs (#275)
- extract module description without @module and across split header comments (#274)
- drop source links for external dependency symbols (#270)
- carry module-level @module description through to generated output (#268)
- deploy docs from void root
- restore Bun.markdown row in PR benchmark (#258)
- remove needless raw string hashes in tabs tests

### Performance

- debug-build NAPI smoke and cache rendering browsers (#263)
- lazily bucket members and drop format! in pure markdown renderer (#262)
- search runtime (#241)
- borrow frontmatter content and move the autolink patterns (#235)
- resolve spellcheck issue lines via binary search (#236)
- gate text autolinking on the cached autolink_index (#234)
- drop redundant allocations in slugify and the YouTube embed (#233)
- cut per-symbol allocations on the generation path (#232)
- tighten leaf/list/fenced block scans (#231)
- SIMD-accelerate inline scanning with memchr (#230)
- hoist per-page constant work out of the page loop (#227)
- memoize per-doc scopes and the prefix-scan vocabulary (#228)
- build the autolink first-byte index once per render (#225)
- cache sort keys and bucket members lazily (#226)
- skip redundant block dispatch on a paragraph's first line (#222)
- SIMD-accelerate the static embed transforms (#224)

### Refactoring

- split html renderer modules (#266)

### Documentation

- expand built-in feature and Void deploy guides (#269)
- format generated API reference
- refresh generated API reference
- add an MDX & Components guide (#240)
- add JSDoc API-docs and i18n guides (#239)
- document dark mode, embed slots, social icons, custom CSS (#238)

## [2.36.0] - 2026-05-31

### Bug Fixes

- render pure markdown sections as sequential headings instead of bold paragraphs (#275)

## [2.35.0] - 2026-05-31

### Bug Fixes

- extract module description without @module and across split header comments (#274)

## [2.34.0] - 2026-05-31

### Features

- add vitest docs test harness (#271)
- add opt-in type parameter docs (#272)

## [2.33.0] - 2026-05-31

### Bug Fixes

- drop source links for external dependency symbols (#270)

## [2.32.0] - 2026-05-31

### Features

- add opt-in content transforms (#265)

### Bug Fixes

- carry module-level @module description through to generated output (#268)
- deploy docs from void root

### Performance

- debug-build NAPI smoke and cache rendering browsers (#263)

### Refactoring

- split html renderer modules (#266)

### Documentation

- expand built-in feature and Void deploy guides (#269)
- format generated API reference
- refresh generated API reference

## [2.31.0] - 2026-05-31

### Features

- add opt-in content transforms (#265)

### Bug Fixes

- deploy docs from void root

### Performance

- debug-build NAPI smoke and cache rendering browsers (#263)

## [2.30.0] - 2026-05-30

### Performance

- lazily bucket members and drop format! in pure markdown renderer (#262)

## [2.29.0] - 2026-05-30

### Features

- add pure markdown render mode via renderStyle option (#261)

## [2.28.0] - 2026-05-30

### Features

- package-manager install tabs with opt-in synced groups (#257)

### Bug Fixes

- restore Bun.markdown row in PR benchmark (#258)

## [2.27.0] - 2026-05-30

### Bug Fixes

- remove needless raw string hashes in tabs tests

### Performance

- search runtime (#241)
- borrow frontmatter content and move the autolink patterns (#235)
- resolve spellcheck issue lines via binary search (#236)
- gate text autolinking on the cached autolink_index (#234)
- drop redundant allocations in slugify and the YouTube embed (#233)
- cut per-symbol allocations on the generation path (#232)
- tighten leaf/list/fenced block scans (#231)
- SIMD-accelerate inline scanning with memchr (#230)
- hoist per-page constant work out of the page loop (#227)
- memoize per-doc scopes and the prefix-scan vocabulary (#228)
- build the autolink first-byte index once per render (#225)
- cache sort keys and bucket members lazily (#226)
- skip redundant block dispatch on a paragraph's first line (#222)
- SIMD-accelerate the static embed transforms (#224)

### Documentation

- add an MDX & Components guide (#240)
- add JSDoc API-docs and i18n guides (#239)
- document dark mode, embed slots, social icons, custom CSS (#238)

## [2.26.0] - 2026-05-29

### Performance

- port the tabs embed transform to Rust (#221)
- port the YouTube embed transform to Rust (#220)
- skip no-op rehype round-trips and redundant per-page work (#218)
- skip non-URL text in autolink scan via memchr (#217)

## [2.25.0] - 2026-05-29

## [2.24.0] - 2026-05-29

## [2.23.0] - 2026-05-29

### Features

- complete typedoc path strategy support (#214)
- autolink bare URLs in text (#205)

### Bug Fixes

- support typedoc markdown paths (#209)
- render JSDoc inline links (#204)

## [2.22.0] - 2026-05-28

### Features

- scaffold crate (#196)

### Bug Fixes

- document local entrypoint exports (#199)

### Refactoring

- split long implementations (#203)

### Documentation

- update benchmarks

## [2.21.0] - 2026-05-28

### Performance

- optimize release profile (#202)

## [2.20.0] - 2026-05-27

### Features

- preview HMR push channel (#192)
- on-save LSP sidecar with opt-in command override (#197)
- component name + attribute completion via registry (#195)
- new crate, LSP diagnostics, CLI (#193)
- asset path completion inside link/image openers (#194)
- support clean URLs in generated Markdown links (#187)

### Bug Fixes

- extract external re-export docs (#198)

### Performance

- add Allocator::for_source_len, use it across LSP + NAPI (#190)
- fast-path text in inline dispatch, pre-size heading scratch (#188)
- reuse parsed list lines (#184)
- scan safe urls in chunks (#183)
- write numeric attrs without strings (#182)
- avoid unused link url allocations (#181)
- write duplicate toc id suffixes in place (#180)
- avoid duplicate toc slug clones (#179)
- avoid inline toc entry clones (#178)
- avoid temporary table row allocations (#177)
- write heading id directly to output, skip callout alloc (#173) (#174)

### Refactoring

- move VitePress frontmatter normalization to Rust (#186)
- move docs generation output into rust (#185)

### Documentation

- add editor extension roadmap (#189)

## [2.19.0] - 2026-05-26

### Features

- support clean URLs in generated Markdown links (#187)
- add allocation and timing profiling mode (#163)
- expose and render API members (#160)
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

- remove panic-prone runtime paths (#171)
- improve docs hero, search, and source links (#169)
- render docs assets in CI
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

- reuse parsed list lines (#184)
- scan safe urls in chunks (#183)
- write numeric attrs without strings (#182)
- avoid unused link url allocations (#181)
- write duplicate toc id suffixes in place (#180)
- avoid duplicate toc slug clones (#179)
- avoid inline toc entry clones (#178)
- avoid temporary table row allocations (#177)
- write heading id directly to output, skip callout alloc (#173) (#174)
- arena strings, dispatch cache, fewer heading allocs (#172)
- byte-level fast paths and zero-copy hot spots (#164)
- batch-parse JSDoc comments in extractor (#111)
- reduce search query allocations (#97)
- speed up markdown render benchmark (#55)

### Refactoring

- move VitePress frontmatter normalization to Rust (#186)
- move docs generation output into rust (#185)
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

- separate user guide and advanced docs
- expand performance documentation (#167)
- update architecture overview (#166)
- add community credits (@kazupon)
- add security policy (#126)
- add contributing guide (#127)
- publish md4x benchmark results (#54)

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
