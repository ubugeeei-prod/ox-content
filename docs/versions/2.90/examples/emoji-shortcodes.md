---
title: Emoji Shortcodes
description: Expand colon shortcodes to Unicode emoji.
---

# Emoji Shortcodes

Emoji shortcodes are opt-in and expand outside fenced and inline code. The
built-in table covers the common GitHub-style shortcode set, and custom values
override built-ins.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      emojiShortcodes: {
        custom: {
          shipit: "ship it",
        },
      },
    }),
  ],
};
```

```md
Ship it :rocket: :shipit:
Status :white_check_mark: :warning: :x:
Faces :smile: :joy: :thinking:
```

Unknown shortcodes are left unchanged.
