---
title: Images
description: Opt-in figures, captions, lazy loading, and safe width/height attributes.
---

# Images

Markdown images stay on the default renderer until you opt in. Enable `images`
to add lazy loading, turn title text into a caption, and accept safe
`width` / `height` attributes.

| Option   | Type                       | Default |
| -------- | -------------------------- | ------- |
| `images` | `boolean` / `ImageOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      images: true,
    }),
  ],
};
```

`true` and `{}` both enable the defaults. The only extra knob is `lazy`
(default `true`):

```ts
oxContent({
  images: { lazy: false },
});
```

## Lazy images

A Markdown image without a title becomes an `<img>` with `loading="lazy"`:

```md
![Diagram](/architecture.png)
```

![Ox Content](/logo-icon.svg)

## Captions

Title text is the caption. The image is wrapped in
`<figure class="ox-figure">` and the title is escaped into `<figcaption>`:

```md
![Diagram](/architecture.png "The transform pipeline")
```

![Ox Content](/logo-icon.svg "The Ox Content mark")

## Dimensions

Optional trailing `{width=N height=M}` is consumed by this feature. It does
not require `attrs`. Only unsigned integers are accepted; anything else is
dropped.

```md
![Diagram](/architecture.png){width=320 height=180}
```

## Safety

Alt text, captions, and `src` values are HTML-escaped. Destinations that use
`javascript:`, `data:`, `vbscript:`, or a protocol-relative `//` do not emit
an `<img src>` with that value. Images inside fenced code, indented code, and
inline code are left untouched.

## Related

- [Syntax Extensions](./syntax-extensions.md)
- [Built-in Features overview](../built-in-features.md)
