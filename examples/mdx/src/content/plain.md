---
title: Plain Markdown
description: Sibling .md stays GFM — JSX, ESM, and expressions are not parsed
---

# Plain Markdown

This sibling `.md` file stays CommonMark + GFM. MDX is **off** unless you set
`mdx: true`. The following lines are **not** JSX, ESM, or expressions.

<NoteCard title="not an island"></NoteCard>

Hello {name}.

import Chart from './Chart.js'

You should **not** see `data-ox-island` on this page. `{name}` stays visible
as source text, and the `import` line is ordinary prose.

## Contrast

- [Built-in MDX (`.mdx`)](./index.mdx)
- [Static HTML vs islands](./islands.mdx)
