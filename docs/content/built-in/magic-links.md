---
title: Magic Links
description: Opt-in rich links for GitHub users, named aliases, and explicit URLs.
---

# Magic Links

Prose often names people, projects, and recurring sites. Ordinary Markdown
links repeat the URL and cannot attach a stable avatar or favicon.
`{link:...}` is opt-in and off by default. The idea is inspired by
[markdown-it-magic-link](https://github.com/antfu/markdown-it-magic-link).

| Option       | Type                           | Default |
| ------------ | ------------------------------ | ------- |
| `magicLinks` | `boolean` / `MagicLinkOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      magicLinks: {
        aliases: {
          Oxc: {
            href: "https://oxc.rs",
            image: "https://github.com/oxc-project.png",
          },
        },
      },
    }),
  ],
};
```

`false` or omitted leaves the source unchanged. `true` or an object enables
the transform. Existing documents stay byte-for-byte identical until a
`{link:...}` form is rewritten.

## Authoring

The form is `{link:BODY}`. `BODY` is one of:

- GitHub user: `{link:@ryoppippi}` — profile URL plus
  `https://github.com/ryoppippi.png`
- GitHub user + label: `{link:@ubugeeei|ox-content}` — same avatar, custom label
- GitHub user + label + URL:
  `{link:@ubugeeei|ox-content|https://github.com/ubugeeei?tab=repositories}`
- Named alias: `{link:Oxc}` — configured `{ href, label?, image? }`
- Explicit label + URL: `{link:Example|https://example.com}` — no image unless
  `favicon` is on

Unknown aliases, unsafe schemes (`javascript:`, `data:`, `file:`), malformed
URLs, and unclosed tags stay literal. Labels and URLs are HTML-escaped.

```md
See {link:@ryoppippi} and {link:Oxc}.
```

See {link:@ryoppippi} and {link:Oxc}.

## Images

GitHub-user forms always use the constructed avatar URL. Aliases use
`image` when set. Explicit URLs have no image unless `favicon` is enabled:

```ts
oxContent({
  magicLinks: {
    favicon: {
      template: "https://icons.duckduckgo.com/ip3/{host}.ico",
    },
  },
});
```

`favicon: true` uses `https://{host}/favicon.ico`. The transform never
fetches; it only writes a URL. `imageOverrides` replace the resolved image
for an exact `href` or a `prefix`.

Output uses stable classes: `ox-magic-link`, `ox-magic-link--github` /
`--alias` / `--url`, `ox-magic-link__image`, and `ox-magic-link__label`.
The image is decorative (`alt=""`); the label is the accessible name.

Custom `ssg: false` hosts should import
`@ox-content/vite-plugin/styles/magic-links.css` (and usually `core.css` for
`--octc-*` tokens). See [Component styles](./component-styles.md).

## Immunity

Fenced, indented, and inline code, raw `<code>` / `<pre>` / `<script>` /
`<style>`, HTML attributes, and already-linked `[text](url)` are not
rewritten.

```md
`{link:@ryoppippi}`
```

`{link:@ryoppippi}`

The walk is a single pass over the source. Aliases are a map lookup, not a
re-parse per name. There is no client JavaScript.

## Related

- [Syntax Extensions](./syntax-extensions.md)
- [Inline Badges](./badges.md)
- [Component styles](./component-styles.md)
- [Built-in Features overview](../built-in-features.md)
- [markdown-it-magic-link](https://github.com/antfu/markdown-it-magic-link)
