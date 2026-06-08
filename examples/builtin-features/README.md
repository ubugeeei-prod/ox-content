# Built-in Feature Examples

Small source examples for built-in `@ox-content/vite-plugin` behavior.

These files are intentionally tiny. Copy one into a Vite example such as
`examples/ssg-vite`, or compare them when checking how a feature is authored.

## Markdown Inputs

| File                              | Shows                                      |
| --------------------------------- | ------------------------------------------ |
| `content/frontmatter-toc.md`      | Default frontmatter and heading TOC input  |
| `content/github-embed.md`         | Default GitHub repository and source cards |
| `content/open-graph-card.md`      | Default Open Graph link preview cards      |
| `content/youtube-embed.md`        | YouTube iframe shorthand                   |
| `content/tabs.md`                 | Generic tab groups                         |
| `content/package-manager-tabs.md` | Package manager command tabs               |
| `content/mermaid-diagram.md`      | Mermaid code fences                        |
| `content/emoji-shortcodes.md`     | Emoji shortcode expansion                  |
| `content/attributes.md`           | Markdown attribute syntax                  |
| `content/wiki-links.md`           | Obsidian-style wiki links                  |
| `content/cjk-emphasis.md`         | Emphasis next to CJK text                  |
| `content/code-annotations.md`     | Highlight, warning, and error line marks   |
| `content/code-imports.md`         | Source snippet imports                     |
| `content/edit-this-page.md`       | Edit link metadata                         |

## Config Snippets

| File                 | Shows                                  |
| -------------------- | -------------------------------------- |
| `config/defaults.ts` | Default parse, TOC, search, and embeds |
| `config/opt-in.ts`   | Common opt-in authoring features       |
| `config/embeds.ts`   | Built-in embed configuration           |
| `client/search.ts`   | Virtual search module usage            |
| `src/example.ts`     | Source file used by `code-imports.md`  |
