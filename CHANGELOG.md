# Changelog

## [3.0.0-alpha.9] - 2026-08-25

### Features

- add typo-tolerant local matching (#868)

### Bug Fixes

- preserve attrs on inline links and transformed images (#935)
- expand MDX parser and renderer edge-case coverage (#894)
- snapshot framework MDX island outputs (#852)
- soften code copy button (#920)
- avoid empty table stretch columns (#919)
- keep block embeds out of generated paragraphs (#936)
- restore typed hover overlays (#918)
- contain mobile menu scrolling (#915)
- highlight WebContainer examples as MDX (#914)
- keep browser cancellation responsive (#870)

### Performance

- skip PR benchmarks for test-only package edits (#851)

## [3.0.0-alpha.8] - 2026-08-25

### Bug Fixes

- skip prerelease marketplace publish (#865)

## [3.0.0-alpha.7] - 2026-08-25

### Performance

- report build and rendered output benchmarks (#862)

## [3.0.0-alpha.6] - 2026-08-25

### Features

- render semantic ordered footnotes (#824) (#850)
- add configurable rich magic links (#825) (#848)
- publish Markdown source companion files (#826) (#849)
- add visible heading permalinks (#829) (#847)
- aggregate external feeds into the blog index (#827) (#846)
- persist Open Graph metadata across builds (#823) (#844)
- include Git lastmod timestamps in sitemap.xml (#843)
- honor YouTube start times (#822) (#842)
- filter the dialog by language and version (#838)
- add a full-fidelity static card with sveltweet visual parity (#833) (#839)
- render quoted posts and reply metadata (#835)
- render video and animated GIF media (#832) (#836)

### Bug Fixes

- hide SIMD nibble tables on non-vector targets (#830)

### Performance

- deduplicate identical page resources by content (#828) (#845)

### Documentation

- add react-tweet and sveltweet license notices (#840) (#841)

## [3.0.0-alpha.5] - 2026-08-25

### Features

- add build-time page-head API and built-in SEO (#821)
- deduplicate identical page resources by content (#828)

## [3.0.0-alpha.4] - 2026-08-25

### Features

- add native TOML and WGSL grammars (#817)

### Bug Fixes

- keep tsgo external and find payloads after wrap (#815)
- typeset multiline display math blocks (#816)
- keep successful Rust runs on the stdio tab (#813)
- keep Code Play Run clear of the copy control (#814)
- decode numeric HTML entities in play fences (#812)

### Refactoring

- drop leftover shiki names from highlight markup (#818)

## [3.0.0-alpha.3] - 2026-08-25

### Bug Fixes

- make vpr release alpha work under Node (#811)
- handle optional and rest tuple types (#810)

## [3.0.0-alpha.2] - 2026-08-25

### Features

- add file-tree icons and polish GitHub, tabs, and cards (#806)
- typeset $…$ / $$…$$ with optional KaTeX (#805)
- resolve per-document component imports for islands (#789) (#804)
- add opt-in git contributors (#797)
- add opt-in blog index authors tags and archive (#791)
- export AST metadata from Vite modules (#795)
- generate opt-in section index pages (#790)
- add opt-in page resources and image processing (#798)
- emit opt-in JSON-LD structured data (#794)
- add opt-in typed hover overlays for TypeScript fences (#792)
- add opt-in PWA manifest and service worker (#793)
- drive framework islands from the MDX AST (#660) (#801)

### Bug Fixes

- first slice of input-triggered panic hardening (#799)
- keep @types/vscode aligned with engines.vscode

### Documentation

- mark shipped 3.0 built-ins and align theme peers (#803)
- match Japanese guides to English depth (#800)
- add built-in MDX example and clarify defaults (#796)

## [3.0.0-alpha.1] - 2026-08-25

### Upgrade Notes

- First 3.0 alpha. npm packages publish to the `alpha` dist-tag; `latest` stays on 2.90.0. Install with `@ox-content/vite-plugin@alpha`.
- Shiki is gone. `highlightTheme` and `highlightLangs` are removed. `highlight: true` uses tree-sitter only; languages without a grammar stay plain.
- Built-ins, MDX, versioning, and Code Play stay opt-in.
- `@ox-content/code-play` is published for the first time.
- APIs may still change before 3.0.0.

### Features

- enable MDX for .mdx source files (#788)
- expose MDX through bindings and mdast (#787)
- localize sidebar navigation labels (#784)
- add opt-in docs versioning, v2.90 snapshot, and Japanese guides (#754)
- add opt-in taxonomies and related pages (#753)
- add opt-in hosted search provider adapter (#752)
- add opt-in header nav, announcement bar, and page chrome (#750)
- add opt-in team members page (#749)
- add opt-in skip link and print styles (#748)
- add opt-in locale switcher (#746)
- add opt-in breadcrumbs (#747)
- render markdown children inside JSX islands (#745)
- add opt-in RSS, Atom, and JSON feeds (#734)
- add opt-in custom 404 page (#733)
- add opt-in redirects, aliases, and path rewrites (#732)
- add opt-in permalinks and frontmatter cascade (#743)
- add opt-in file tree blocks (#724)
- serialize XSS-safe island props from JSX attributes (#744)
- add opt-in `$` / `$$` math (#717)
- add opt-in draft, unlisted, and scheduled pages (#739)
- add opt-in card and link-card blocks (#726)
- parse document-level flow and text expressions (#741)
- parse JSX fragments, spreads, and member names (#736)
- add opt-in step lists (#727)
- parse import and export as MdxjsEsm (#729)
- add opt-in copy, external-link, and back-to-top chrome (#725)
- add opt-in figures and lazy images (#720)
- add opt-in sitemap, robots.txt, and llms.txt (#723)
- parse JSX elements when mdx is enabled (#722)
- add opt-in inline badges (#719)
- add opt-in Markdown file includes (#718)
- make the right-hand page outline opt-in (#715)
- add opt-in ::: tip custom containers (#707)
- add opt-in previous/next page links (#713)
- add AST nodes and parser option (#686)
- hide dead SSG Typecheck and expose Cancel (#712)
- drop Shiki and highlight with tree-sitter only (#710)
- prove SSG hydrate+Run in CI and add session.cancel() (#648) (#711)
- run browser JavaScript in a sandboxed iframe (#705)
- emit a self-contained auto-hydrating browser client (#703)
- harden playground proxies and document endpoints (#663)
- dedicated stderr viewer and stdout/stderr API (#662)
- add opt-in plugin for on-demand sample execution (#649)

### Bug Fixes

- preserve documentation version in sidebar navigation (#785)
- stabilize MPA navigation (#783)
- enforce generated link integrity (#782)
- keep default surfaces flat (#781)
- make code copy an interaction icon (#780)
- compact mobile breadcrumb spacing (#779)
- harden mobile menu interactions (#777)
- restore mobile content gutters (#776)
- eliminate doubled table borders (#773)
- keep copy control clear of code (#772)
- align nested sidebar hierarchy (#771)
- keep locale nav current and theme header selects (#758)
- split header-chrome tests under the file line limit (#757)
- skip landing-page chrome and honor announcement dismiss (#756)
- regenerate stale API reference pages (#755)
- split locale-switcher Vite tests under the file line limit (#751)
- keep generated declarations in NAPI sort order (#728)
- keep steps preamble and refresh NAPI declarations (#737)
- keep CI NAPI JSDoc newline encoding in API examples (#721)
- build Code Play before Void docs deploy (#716)

### Performance

- skip GFM autolink rewrite when a block has no bare URL (#643)
- gate default autolinks with a reused :// finder (#642)
- scan pre-pass newlines with 32-byte NEON (#641)

### Documentation

- fold remaining security notes onto main (#648) (#704)
- add Ox Content 3.0 and docs-site feature roadmaps (#708)
- expand the guide and add a standalone example (#697)

## [2.90.0] - 2026-08-23

### Features

- highlight markdown, block and inline (#619)
- expose the tree-sitter highlighter to JavaScript (#615)
- cover python, go, java, c, c++ and yaml (#614)
- add a tree-sitter syntax highlighter (#613)

### Bug Fixes

- sort JSDoc tag keys so checked-in docs.json is stable (#640)
- keep type cross-reference links after highlighting (#638)
- emit HTML attribute names, not React prop names (#637)
- serve every page that asks for the bindings at once (#632)
- stop truncating generated examples at their first blank line (#626)

### Performance

- intern paragraph-sized nodes so Text cells pack at 32 bytes (#636)
- scan inline markers 32 bytes at a time with overlapping tails (#635)
- reserve the document block list from source density (#634)
- intern large AST nodes so the common cells pack densely (#633)
- highlight a page off the main thread (#631)
- spread a page's distinct snippets across threads (#630)
- load only the grammars a page's pending blocks name (#628)
- highlight one exotic block without surrendering the page (#625)
- highlight a repeated snippet once per page (#624)
- read through the links inside a member type (#623)
- highlight a page's code without an HTML round trip (#620)
- render plain-text blocks natively (#617)
- highlight natively where a grammar exists (#616)

### Documentation

- regenerate the checked-in API reference and fail CI on drift (#639)
- record the MSRV bump in the v2.89.0 notes

## [2.89.0] - 2026-08-22

### Upgrade Notes

- **The Rust crates now require Rust 1.95.0 and build on edition 2024** (#607).
  The previously declared MSRV of 1.83.0 had not been buildable for some time —
  `oxc_parser`, `oxc_allocator` and `oxc_ast` all require 1.95.0 — so this
  corrects the declaration rather than raising the real requirement. The npm
  packages are unaffected.

### Features

- complete bare mode and let ssg.render take a theme component (#612)

### Bug Fixes

- stop inlining package imports into .ts templates (#611)

### Performance

- copy short escape runs without memmove (#610)
- stop walking the arena AST to drop it (#606)

## [2.88.0] - 2026-08-22

### Bug Fixes

- generate OG images in bare mode (#605)
- export the ./jsx-runtime subpath (#604)
- stop rewriting other origins' .md URLs (#603)

### Performance

- reuse first list dispatch result (#599)
- reject impossible prepass candidates (#597)
- reuse parsed list siblings (#596)
- fast-path plain inline text (#595)
- eliminate default-path allocations (#593)

## [2.87.0] - 2026-08-21

### Features

- composable theme presets — 27 skins × 45 colour schemes (#589)

### Bug Fixes

- make build:npm reach the nested theme packages (#590)
- satisfy the two lints Rust 1.98 turned on (#591)

### Documentation

- generate the gallery at build time, and show every preset (#592)

## [2.86.0] - 2026-08-18

### Performance

- classify escape bytes with a SIMD nibble lookup (-8% render) (#588)
- classify inline markers with a SIMD nibble lookup (-9% parse) (#587)
- gate URL autolinking on `://` instead of `:` (-7% render) (#586)
- stop re-probing for a table on every table row (-13% on the span) (#585)
- fuse the code-span newline probe into the closer scan (-3% parse) (#584)
- skip fenced-code interiors in the pre-pass (-3% parse) (#583)
- cover the escape tail with an overlapping word read (-4% render) (#582)
- let the table probe hand back the line end (-2% parse) (#581)
- reject autolink candidates on `://` instead of `:` (-4% parse) (#579)
- reserve inline children from the content length (-4% parse) (#577)
- scan pre-pass lines with SWAR instead of memchr (-10% on the pass) (#576)

### Refactoring

- move the inline reserve heuristic beside push_text (#578)

## [2.85.0] - 2026-08-18

### Performance

- gate URL autolinking on `://` instead of `:` (-7% render) (#586)
- stop re-probing for a table on every table row (-13% on the span) (#585)
- fuse the code-span newline probe into the closer scan (-3% parse) (#584)
- skip fenced-code interiors in the pre-pass (-3% parse) (#583)
- cover the escape tail with an overlapping word read (-4% render) (#582)
- let the table probe hand back the line end (-2% parse) (#581)
- reject autolink candidates on `://` instead of `:` (-4% parse) (#579)
- reserve inline children from the content length (-4% parse) (#577)
- scan pre-pass lines with SWAR instead of memchr (-10% on the pass) (#576)

### Refactoring

- move the inline reserve heuristic beside push_text (#578)

## [2.84.0] - 2026-08-17

### Performance

- walk short inline runs before the chunked scan (+8-16% on the routine) (#575)
- scan for escapes with SWAR word tests (+40-67% on the routine) (#573)
- key the heading-id map by CompactString (-43% allocations) (#572)

### Documentation

- refresh the benchmark tables and the ratios quoted around them (#574)

## [2.83.0] - 2026-08-17

### Features

- add detail-tier micro-spans with guard-cost calibration (#562)

### Bug Fixes

- stop the gfm option from disabling its own sub-features (#567)

### Performance

- reuse arena and renderer per call, return plain objects (#570)
- gate the URL-autolink scan on a required byte (-23% render on prose) (#568)
- re-enable wasm-opt with the post-MVP feature set (-7% module size) (#566)
- fold soft line breaks into the running text node (#565)
- gate and deduplicate the GFM autolink post-pass (#563)
- win back the md4x benchmark on small documents (#560)

## [2.82.1] - 2026-08-13

### Bug Fixes

- publish compatible Vite peer ranges

## [2.82.0] - 2026-08-13

### Features

- make cjkEmphasis pair runs against CJK punctuation (#545) (#548)
- add SolidJS integration (#540) (#546)
- add boundary-free ox-content rows to the native competitor runner (#524)

### Performance

- chunk-scan JSON string escaping and drop per-number allocations (#522)

### Documentation

- measure and publish CommonMark conformance per engine (#545) (#547)
- publish TypeScript renderer results (#539)
- expand built-in features into per-feature guides with live examples (#527)
- refresh benchmark numbers (#526)
- retire the "cargo doc for JavaScript" tagline (#525)

## [2.81.0] - 2026-07-18

### Features

- add native competitor rows (pulldown-cmark, Grok Build markdown) (#513)
- span the parser pre-passes and inline post-passes (#511)

### Performance

- reject autolink candidates on the second byte before prefix checks (#520)
- pre-size the arena from the source length (#519)
- skip the pre-pass line scan when the source has no "]:" (#517)
- close HTML blocks with whole-block searches instead of per-line scans (#515)
- fuse the reference and footnote pre-passes behind first-byte dispatch (#512)

### Refactoring

- split native-competitors main.rs under file line limit (#514)

## [2.80.0] - 2026-07-18

### Features

- implement GFM footnotes (#507)
- implement the GFM tagfilter extension (#506)
- GFM autolink extension and extension spec suite (#505)
- complete HTML block start conditions (#500)
- lazy continuation for block quotes (#499)
- support link reference definitions (#494)
- decode entity and numeric character references (#493)
- support CommonMark autolinks (#492)
- support indented code blocks (#488)
- support setext headings (#487)

### Bug Fixes

- stop hanging on whitespace-only input (#509)
- reach full CommonMark 0.31.2 conformance (#504)
- list looseness, marker indent limits, deep laziness (#503)
- preserve tab stops when stripping list and quote markers (#502)
- enforce the spec grammar for inline raw HTML (#501)
- forbid nested links and flatten image alt text (#498)
- ATX heading details, fence info rules, hr precedence (#497)
- align list item structure with CommonMark (#496)
- implement emphasis via the CommonMark delimiter stack (#495)
- parse code spans with multi-backtick delimiters (#491)
- render tight lists without paragraph wrappers (#490)
- handle hard and soft line break whitespace (#489)
- parse inline link destinations and titles per spec (#486)
- stop panicking on backslash before multibyte chars (#485)

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
