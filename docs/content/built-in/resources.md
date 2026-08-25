---
title: Page resources and image processing
description: Opt-in page-bundle assets with build-time resize, crop, and format transforms.
---

# Page resources and image processing

When `resources` is enabled, each Markdown page directory is a **bundle**.
Images that sit next to the page (or in a subdirectory of that directory)
become addressable with relative URLs. Optional query-string transforms
resize, crop, or convert those files at build time.

The feature is off unless you turn it on. Existing sites stay unchanged.
`images` (figures, captions, lazy-loading) is a separate option and stays
compatible.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      resources: true,
    }),
  ],
};
```

`false` or omitted keeps colocated assets and transforms off. `true` or `{}`
enables the defaults. An object enables the feature and overrides only the
fields you set:

```ts
oxContent({
  resources: {
    formats: ["png", "jpeg", "webp"],
    widths: [400, 800],
    missing: "error",
    dedupe: false,
  },
});
```

| Option      | Type                           | Default                   |
| ----------- | ------------------------------ | ------------------------- |
| `resources` | `boolean` / `ResourcesOptions` | `false`                   |
| `formats`   | `string[]`                     | `["png", "jpeg", "webp"]` |
| `widths`    | `number[]`                     | `[]` (any positive width) |
| `missing`   | `"error"` / `"warn"`           | `"error"`                 |
| `dedupe`    | `boolean`                      | `false`                   |

## Page bundle

The bundle root is the directory that contains the Markdown file.

```
content/
  guide.md
  hero.png
  posts/
    hello.md
    cover.png
```

From `guide.md`, `![Hero](./hero.png)` and `![Hero](hero.png)` resolve to
`content/hero.png`. From `posts/hello.md`, `./cover.png` resolves to
`content/posts/cover.png`. Nested files under the same page directory are
also inside the bundle.

SSG copies the file next to the generated HTML so the relative URL keeps
working in the output tree.

## Transforms

Append a query string to request a build-time derivative:

```md
![Wide](./hero.png?width=800)
![Fill](./hero.png?width=800&height=400&crop=center)
![Jpeg](./hero.png?width=400&format=jpeg)
```

Use a `<destination>` when the query string contains `&`, so Markdown does
not treat the rest of the URL as text.

| Param          | Meaning                                             |
| -------------- | --------------------------------------------------- |
| `width` / `w`  | Target width in pixels                              |
| `height` / `h` | Target height in pixels                             |
| `crop=center`  | Scale to cover `width` × `height`, then center-crop |
| `crop=x,y,w,h` | Crop that rectangle from the source                 |
| `format`       | Output container: `png`, `jpeg` / `jpg`, or `webp`  |

When only one of `width` or `height` is set, the other side follows the
source aspect ratio. `crop=center` requires both `width` and `height`.

Pixel transforms encode **PNG** and **JPEG**. `webp` is copied when the
source is already WebP and no pixel transform is requested. If `widths` is
non-empty, `?width=` must be one of those values. `?format=` must be in
`formats`.

## Cache

Each derivative is cached under `.cache/ox-content-resources/` in the
project root. The cache key is SHA-256 of:

- the absolute source path
- the source file mtime
- the normalized transform (`width`, `height`, `crop`, `format`)

A later build with the same key copies the cached bytes instead of
re-encoding. Changing the source file or any transform param produces a new
key and a new output filename.

## Content deduplication

`resources.dedupe` is **off by default**. `true` and `{}` do not turn it on.
Set `dedupe: true` to write identical emitted bytes once:

```ts
oxContent({
  resources: { dedupe: true },
});
```

The digest is SHA-256 of the **final emitted bytes**, a NUL, and the serving
extension (`jpg` for JPEG). The canonical file is:

`/assets/content/<sha256>.<ext>`

`base` is prefixed (`/docs/` → `/docs/assets/content/...`). HTML `src`,
`poster`, and relevant `href` values that point at the resource become that
URL. Leftover query strings (after consumed transform params) and hash
fragments stay on the rewritten URL. Remote, `data:`, and `javascript:`
values are not rewritten.

The first page that produces a digest writes the canonical file. Later pages
reuse that path and hash. Deduping does not decode images. Large files are
hashed incrementally.

The original page-output path is kept as an alias: a hard link when the
filesystem allows, otherwise a copy. A failed link never overwrites a
shared inode. Names are deterministic across builds.

Same bytes with a different extension stay separate. Different bytes stay
separate.

## Missing sources

When a relative image is missing, the default `missing: "error"` **fails
the build** (`PageResourceError`). Set `missing: "warn"` to keep the page
and record the issue on the SSG result instead.

```ts
oxContent({
  resources: { missing: "warn" },
});
```

## Path escape

After resolve, the source must stay inside **both** the page bundle and
`srcDir`. `../` that leaves the page directory is rejected and fails the
build. Absolute filesystem paths, `javascript:`, `data:`, `vbscript:`,
protocol-relative `//`, and site-root `/...` URLs are not processed as
page resources.

Image syntax inside fenced code, indented code, and inline code is not an
`<img>` and is left untouched.

Rewritten `src` values are HTML-escaped. Output names come from the
resource basename plus a cache-key suffix, never from raw author input.

## Related

- [Images](./images.md)
- [Site Generation](./site-generation.md)
- [Built-in Features overview](../built-in-features.md)
