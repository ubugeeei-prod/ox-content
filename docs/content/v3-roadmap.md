---
title: Ox Content 3.0 Roadmap
description: Release tracker for theme packages, complete MDX, Code Play, opt-in built-ins, and tree-sitter highlighting.
---

# Ox Content 3.0 Roadmap

Tracking issue: [#699](https://github.com/ubugeeei-prod/ox-content/issues/699).

3.0 is the release that graduates experimental surfaces and accepts the
highlighting breaking change. New Markdown and extra chrome stay **opt-in**.
Work lands in small conventional PRs with failing tests first.

| Pillar                      | Issue                                                          | Notes                                                                                                    |
| --------------------------- | -------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| Stable theme packages       | [#700](https://github.com/ubugeeei-prod/ox-content/issues/700) | Skins and color schemes become an official contract                                                      |
| Complete MDX support        | [#701](https://github.com/ubugeeei-prod/ox-content/issues/701) | JSX, imports, markdown-in-JSX; islands stay JS-free by default                                           |
| Code Play                   | [#648](https://github.com/ubugeeei-prod/ox-content/issues/648) | Linked pillar; see the [Code Play Roadmap](./code-play-roadmap.md)                                       |
| Built-in docs-site features | [#650](https://github.com/ubugeeei-prod/ox-content/issues/650) | Authoring, site outputs, theme chrome — default OFF                                                      |
| Tree-sitter highlighting    | [#702](https://github.com/ubugeeei-prod/ox-content/issues/702) | Landed in [#710](https://github.com/ubugeeei-prod/ox-content/pull/710); `highlight: true` is native only |

Also tracked elsewhere and not duplicated here:

- i18n / MF2 core: [#451](https://github.com/ubugeeei-prod/ox-content/issues/451)

## Breaking changes

- `highlightTheme` and `highlightLangs` go away. There is one highlighter.
- Languages with no tree-sitter grammar stay plain.
- Theme package peer ranges move to 3.x.
- Built-ins and Code Play do **not** turn on just because the major version
  changed. Each still needs an explicit install or option.

Syntax token CSS (`--octc-shiki-*`, `class="shiki"`) stays so published color
packages keep working. Those names are historical.

## Built-ins

The feature list lives on the [Docs Site Feature Roadmap](./docs-site-feature-roadmap.md)
and in [#650](https://github.com/ubugeeei-prod/ox-content/issues/650).
