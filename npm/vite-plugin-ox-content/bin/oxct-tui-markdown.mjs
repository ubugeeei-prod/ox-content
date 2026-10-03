import { marked } from "marked";
import { clean, plain, wrap, width, pad } from "./oxct-tui-style.mjs";

export function renderMarkdown(source, columns, s) {
  source = clean(source).replaceAll("\r\n", "\n").replaceAll("\t", "    ");
  const lines = [],
    outline = [],
    links = [];
  const frontmatter = source.match(/^---\n([\s\S]*?)\n(?:---|\.\.\.)\s*(?:\n|$)/);
  const put = (text = "", indent = "") => {
    for (const line of wrap(text, columns - width(indent))) lines.push(indent + line);
  };
  const codeLine = (line, language) => {
    if (!language) return s.code(line);
    return line.replace(
      /(\/\/.*|# .*$)|("(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`[^`]*`)|(\b\d+(?:\.\d+)?\b)|(\b(?:const|let|var|function|class|return|import|from|export|default|async|await|if|else|for|while|true|false|null|undefined|def|self|fn|pub|use|mut|impl|struct|enum|match|type|interface)\b)/g,
      (token, comment, string, number, keyword) =>
        comment
          ? s.dim(token)
          : string
            ? s.code(token)
            : number
              ? s.warning(token)
              : keyword
                ? s.accent(token)
                : token,
    );
  };
  const inline = (tokens = []) =>
    tokens
      .map((token) => {
        if (token.type === "strong") return s.bold(inline(token.tokens));
        if (token.type === "em") return s.italic(inline(token.tokens));
        if (token.type === "del") return s.dim(inline(token.tokens));
        if (token.type === "codespan") return s.code(token.text);
        if (token.type === "br") return "\n";
        if (token.type === "html") return "";
        if (token.type === "image") return s.muted(`[image: ${token.text}]`);
        if (token.type === "link") {
          const label = inline(token.tokens);
          links.push({ title: plain(label), href: token.href });
          return s.accent(label) + s.dim(` (${token.href})`);
        }
        return token.tokens
          ? inline(token.tokens)
          : (token.text ?? token.raw ?? "").replaceAll("\n", " ");
      })
      .join("");
  const blocks = (tokens, indent = "") => {
    for (const token of tokens) {
      if (token.type === "space") continue;
      if (token.type === "heading") {
        const title = inline(token.tokens);
        outline.push({ title: plain(title), depth: token.depth, line: lines.length });
        put(s.accent(s.bold(`${"#".repeat(token.depth)} ${title}`)), indent);
        if (token.depth === 1)
          put(s.muted("─".repeat(Math.max(1, columns - width(indent)))), indent);
        put();
      } else if (token.type === "paragraph" || token.type === "text") {
        put(inline(token.tokens) || token.text, indent);
        put();
      } else if (token.type === "code") {
        put(s.muted(`╭─ ${token.lang || "code"}`), indent);
        for (const line of token.text.split("\n"))
          put(codeLine(line, token.lang), indent + s.muted("│ "));
        put(
          s.muted("╰" + "─".repeat(Math.max(1, Math.min(columns - width(indent) - 1, 48)))),
          indent,
        );
        put();
      } else if (token.type === "blockquote") {
        blocks(token.tokens, indent + s.muted("│ "));
      } else if (token.type === "list") {
        token.items.forEach((item, index) => {
          const prefix = item.task
            ? item.checked
              ? "☑ "
              : "☐ "
            : token.ordered
              ? `${Number(token.start) + index}. `
              : "• ";
          const start = lines.length;
          blocks(item.tokens, indent + " ".repeat(width(prefix)));
          if (lines.length > start)
            lines[start] =
              indent + s.accent(prefix) + lines[start].slice(indent.length + width(prefix));
        });
      } else if (token.type === "table") {
        const rows = [token.header, ...token.rows].map((row) =>
          row.map((cell) => inline(cell.tokens)),
        );
        const available = Math.max(
          rows[0].length,
          columns - width(indent) - rows[0].length * 3 - 1,
        );
        const sizes = rows[0].map((_, index) =>
          Math.max(
            1,
            Math.min(
              Math.floor(available / rows[0].length),
              ...[Math.max(...rows.map((row) => width(row[index])))],
            ),
          ),
        );
        const rule = s.muted("├" + sizes.map((size) => "─".repeat(size + 2)).join("┼") + "┤");
        rows.forEach((row, index) => {
          const cells = row.map((cell, col) => wrap(index === 0 ? s.bold(cell) : cell, sizes[col]));
          const height = Math.max(...cells.map((cell) => cell.length));
          for (let line = 0; line < height; line++)
            put(
              s.muted("│ ") +
                cells.map((cell, col) => pad(cell[line] ?? "", sizes[col])).join(s.muted(" │ ")) +
                s.muted(" │"),
              indent,
            );
          if (index === 0) put(rule, indent);
        });
        put();
      } else if (token.type === "hr") {
        put(s.muted("─".repeat(Math.max(1, columns - width(indent)))), indent);
        put();
      } else if (token.type === "html") {
        const text = token.text.replace(/<[^>]*>/g, "").trim();
        if (text) {
          put(s.dim(text), indent);
          put();
        }
      } else if (token.tokens) blocks(token.tokens, indent);
    }
  };
  if (frontmatter) {
    put(s.muted("◇ Frontmatter"));
    frontmatter[1]
      .split("\n")
      .slice(0, 8)
      .forEach((line) => put(s.dim(line), "  "));
    put();
    source = source.slice(frontmatter[0].length);
  }
  blocks(marked.lexer(source, { gfm: true }));
  return { lines, outline, links };
}
