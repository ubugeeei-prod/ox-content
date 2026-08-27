# TypeScript → Rust migration audit

Answers the audit bullet of #902, split out as #1073. The point is to replace
"move core behavior to Rust where practical" with a list of named slices, each
with a reason, so the migrations after this are scoped rather than
opportunistic.

## Method

Every `.ts` file under `npm/vite-plugin-ox-content/src` that is not a test or a
declaration file — 209 modules. Each was classified from its imports and the
globals it touches:

- **ui-runtime** — reaches browser globals, or is a template string of
  JavaScript emitted into the page.
- **framework-integration** — React / Vue / Svelte / Solid specific.
- **platform-glue** — imports `vite` or Node built-ins, or is a plugin entry
  point.
- **core-behavior** — none of the above: data in, data out.

Duplicates were found by matching each exported TypeScript function against
public Rust `fn` names, camelCase to snake_case, then reading both sides. That
match is a starting point, not a verdict: `lint.ts` looked like a duplicate of
`ox_content_markdown_lint` and is not — it is TypeScript orchestration over
that engine, reaching the binding through a `napiBinding` local rather than the
`importNapiModule` helper the first pass looked for.

## Where the code sits

| Class                 | Modules | Already delegating to Rust |
| --------------------- | ------- | -------------------------- |
| core-behavior         | 124     | 13                         |
| platform-glue         | 58      | 13                         |
| ui-runtime            | 25      | 3                          |
| framework-integration | 2       | 1                          |
| **total**             | **209** | **30**                     |

No module is unclassified.

The number that matters is the first row: 111 modules are pure data-in /
data-out and do not go through Rust. That is not 111 migrations — most are
small option normalizers where a NAPI hop would cost more than the function —
but it is where the duplicates live.

## Duplicates found

Both implementations exist; neither calls the other.

| Rule               | TypeScript                                                           | Rust                                            | State                                    |
| ------------------ | -------------------------------------------------------------------- | ----------------------------------------------- | ---------------------------------------- |
| Feed generation    | `feeds.ts` → `generateFeeds`                                         | `ox_content_ssg::generate_feeds`                | drifted, TS ahead — #1074                |
| Site maps          | `site-maps.ts` (293 lines) → `generateSiteMaps`                      | `ox_content_ssg::site_maps::generate_site_maps` | unverified                               |
| Redirects          | `redirects.ts` (339 lines) → `generateRedirectHtml`, `normalizePath` | `ox_content_ssg::redirects`                     | unverified                               |
| Feed date fallback | `blog-feed-date.ts` → `parseFeedDate`                                | `ox_content_ssg::parse_date`                    | partial: the RFC 822 fallback is TS-only |

Two were already closed while writing this audit, and both had drifted before
anyone noticed:

- `extractVideoId` — #1063. The Rust doc comment said it "mirrors the TS
  extractVideoId"; they agreed, and the copy was deleted.
- The feed date parser — #1068. They did **not** agree. Three inputs differed,
  TypeScript wrong on all three, one of them live.

That is the argument for doing the rest: every duplicate examined so far had
either drifted or was one edit away from it.

## Ranked migrations

1. **Feeds** — #1074. Largest surface, already drifted, and the drift runs
   TS-ahead so the port adds features to Rust rather than moving them.
2. **Site maps** — `generate_site_maps` exists and nothing calls it from the
   plugin. Verify the two agree, then delegate. Small and self-contained.
3. **Redirects** — same shape as site maps: a Rust implementation with no
   caller. `generateRedirectHtml` emits markup, so drift here is user-visible.
4. **Feed date RFC 822 fallback** — push the one remaining branch into
   `ox_content_ssg::parse_date` so `blog-feed-date.ts` disappears too.

None depends on another. Feeds is tracked as #1074; the other three are
small enough to land directly rather than sit in the backlog.

## Staying in TypeScript, deliberately

- **Search runtimes** — `search/hosted-runtime.ts` and `search/local-runtime.ts`
  are template strings of JavaScript emitted into the page. `parseSearchQuery`
  and `search` appear duplicated against Rust and are not a migration
  candidate: they run in the reader's browser. Sharing them would mean wasm in
  the search box, which is a much larger trade than a search box justifies.
- **Trivial helpers** — `escapeHtml`, `escapeXml`, `escapeAttribute`,
  `isSafeDest`, `stripMarkdownExtension` and friends. Real duplicates, but each
  is a few lines and crossing the NAPI boundary costs more than running it.
  Duplication is the cheaper defect here.
- **`lint.ts`** — already Rust-backed. The TypeScript around it normalizes
  options and loads `cspell-lib` for opt-in standard dictionaries, which is a
  JavaScript-ecosystem dependency.
- **Option normalizers** — the bulk of the 111. They translate user config into
  a resolved shape and are consumed immediately by TypeScript.

## Note on measuring

Nothing here reports what a migration costs. #1075 covers that; until it lands,
a migration PR cannot show whether it grew the native artifact or the build.
