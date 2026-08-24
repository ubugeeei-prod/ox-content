---
title: Custom 404 Page
description: Opt-in themed 404 page written next to generated HTML.
---

# Custom 404 Page

When `ssg.notFound` is enabled, the SSG build writes a themed 404 page using
the default layout — navigation, search, and the rest of the site chrome. The
page is omitted from the search index and from `sitemap.xml` / `llms.txt` when
those are enabled.

The feature is off unless you turn it on. Existing sites stay unchanged.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        notFound: true,
      },
    }),
  ],
};
```

`false` or omitted keeps the file off. `true` enables the defaults (`404.md` in
`srcDir`, output `404.html`). An object enables the feature and overrides only
the fields you set:

```ts
oxContent({
  ssg: {
    notFound: {
      source: "pages/missing.md",
      output: "not-found.html",
    },
  },
});
```

| Option         | Type                          | Default      |
| -------------- | ----------------------------- | ------------ |
| `ssg.notFound` | `boolean` / `NotFoundOptions` | `false`      |
| `source`       | `string`                      | `"404.md"`   |
| `output`       | `string`                      | `"404.html"` |

If the source file is missing, the build still writes a themed page titled
"Page not found". Enabling the option always produces the output file.

Titles and other metadata from `404.md` are escaped in the HTML document so
they cannot break out of `<title>` or attributes.

## Related

- [Site Generation](./site-generation.md)
- [Search](./search.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [Built-in Features overview](../built-in-features.md)
