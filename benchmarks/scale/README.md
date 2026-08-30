# Scale benchmark

`bundle-size/` measures four-page apps against other frameworks. That answers
"is a small site fast?", which is not the question a large project asks. The
cost that hurts a large project is the one that grows faster than the page
count, and a four-page fixture cannot show it.

This suite measures growth, and the marginal cost of each thing a build can be
asked to do.

```bash
vp install                       # benchmarks/scale is a workspace package
node benchmarks/scale/run.mjs    # all three suites
node benchmarks/scale/run.mjs --suite build --pages 3200 --json out.json
```

The pages are synthesized by `site.mjs` into `content/` (gitignored) and carry
what a real documentation page carries — frontmatter, headings, prose, fenced
code in six languages, tables, lists, links to sibling pages, images,
containers, footnotes. A corpus of lorem ipsum would measure the wrong thing:
a build spends its time on highlighting, SSG rendering and asset work, not on
parsing paragraphs.

## Suites

### `build` — does the build stay linear?

Runs the same project at 200, 800 and 3,200 pages. Each step is 4x the pages,
so linear is about x4 and quadratic about x16. The number to watch is the
ratio, not the total: a total nobody can calibrate hides a regression that a
ratio makes obvious.

### `og` — what does an OG image cost, cold and warm?

Builds once with an empty cache and once with a full one, then sweeps
`concurrency`. Every run is checked to have produced every image: a build
whose renderer never starts still succeeds — OG failures are reported per page
rather than thrown — and it looks spectacularly fast. Timing a run without
checking what it produced is how a broken renderer reads as a speedup.

Needs Chromium:

```bash
vp exec --filter @ox-content/vite-plugin -- playwright install chromium
```

### `features` — what does each builtin cost?

Measures a baseline with everything off, then turns on one builtin at a time
over the same corpus and reports the delta. Features whose delta is within the
noise floor (a few tens of milliseconds here) are indistinguishable from zero;
the suite exists to find the ones that are not.

## What it found

The suite was written to answer "is v3 ready for a very large project", and
the first run answered no:

- **`siteMaps` cost 25x the entire rest of the build** — 14.9 s against 0.59 s
  on 400 pages. `lastUpdated` ran one `git log` process per page, and a page
  git has never seen cannot be answered without walking the whole history.
  Fixed in [#1231]: one walk for the whole tree, 87x to 564x depending on size.
- **OG images did not parallelize** — pages in one browser share a single
  connection to it, so raising `concurrency` added pages to the same browser
  and the total barely moved. Fixed in [#1232]: 42.7 ms to 8.5 ms per image at
  the default setting.

Everything else measured flat or within noise, including the build itself,
which is sublinear in the page count: 3,200 pages cost 0.50 ms each against
2.30 ms at 200 pages, because fixed setup dominates at that size.

[#1231]: https://github.com/ubugeeei-prod/ox-content/pull/1231
[#1232]: https://github.com/ubugeeei-prod/ox-content/pull/1232

## Runtime

`--suite build` is about a minute. `--suite features` runs one build per
builtin, so about five minutes at the default 400 pages. `--suite og` depends
on Chromium and is a few minutes at 100 pages. Use `--pages` to trade coverage
for time, and `--iterations` for how many runs each measurement takes the best
of.
