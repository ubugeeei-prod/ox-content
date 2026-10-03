import { relative } from "node:path";
import { clean, clip, pad, plain } from "./oxct-tui-style.mjs";

export function layout(columns, rows, sidebar) {
  columns = Math.max(20, columns || 80);
  rows = Math.max(8, rows || 24);
  const sidebarWidth = sidebar && columns >= 80 ? Math.min(30, Math.floor(columns / 4)) : 0;
  return {
    columns,
    rows,
    sidebarWidth,
    contentWidth: columns - sidebarWidth - (sidebarWidth ? 3 : 0) - 4,
    height: rows - 5,
  };
}

export function frame(state, document, files, root, size, s) {
  const { columns, rows, height, sidebarWidth } = size;
  const name = files[state.current] ? clean(relative(root, files[state.current])) : "stdin.md";
  const result = [
    pad(s.accent(s.bold(" ◆ OX CONTENT")) + s.muted("  /  ") + s.bold(name), columns),
    s.muted("─".repeat(columns)),
  ];
  const items =
    state.focus === "outline"
      ? document.outline.map((item) => `${"  ".repeat(Math.min(3, item.depth - 1))}${item.title}`)
      : state.focus === "links"
        ? document.links.map((item) => item.title || item.href)
        : files.map((file) => clean(relative(root, file)));
  const selected =
    state.focus === "outline"
      ? state.outlineIndex
      : state.focus === "links"
        ? state.linkIndex
        : state.fileIndex;
  const paneTitle =
    state.focus === "outline" ? "OUTLINE" : state.focus === "links" ? "LINKS" : "FILES";
  const paneStart = Math.max(0, selected - Math.floor((height - 1) / 2));
  for (let row = 0; row < height; row++) {
    let left = "";
    if (sidebarWidth) {
      if (row === 0) left = s.muted(` ${paneTitle} · ${items.length}`);
      else {
        const index = paneStart + row - 1;
        const label = items[index];
        if (label !== undefined)
          left =
            index === selected
              ? s.selected(pad(` › ${label}`, sidebarWidth))
              : s.dim(`   ${label}`);
      }
      left = pad(left, sidebarWidth) + s.muted(" │ ");
    }
    const index = state.offset + row;
    let line = document.lines[index] ?? "";
    if (!sidebarWidth && state.focus !== "reader") {
      const item = paneStart + row - 1;
      line =
        row === 0
          ? s.muted(`${paneTitle} · ${items.length}`)
          : items[item] === undefined
            ? ""
            : item === selected
              ? s.selected(`› ${items[item]}`)
              : `  ${items[item]}`;
    }
    if (state.query && plain(line).toLocaleLowerCase().includes(state.query.toLocaleLowerCase())) {
      const text = plain(line),
        lower = text.toLocaleLowerCase(),
        needle = state.query.toLocaleLowerCase();
      let cursor = 0,
        marked = "",
        found;
      while ((found = lower.indexOf(needle, cursor)) !== -1) {
        marked += text.slice(cursor, found) + s.selected(text.slice(found, found + needle.length));
        cursor = found + needle.length;
      }
      line = marked + text.slice(cursor);
    }
    result.push(pad(left + "  " + clip(line, size.contentWidth), columns));
  }
  const percent = document.lines.length
    ? Math.min(100, Math.round(((state.offset + height) / document.lines.length) * 100))
    : 100;
  const progress = ` ${state.current + 1}/${files.length || 1} · ${state.offset + 1}/${Math.max(1, document.lines.length)} · ${percent}%`;
  result.push(s.muted("─".repeat(columns)));
  const status = state.searching
    ? s.accent(` / ${state.query}▏  Enter: find · Esc: cancel`)
    : state.message
      ? s.warning(` ${state.message}`)
      : s.muted(progress + ` · ${state.focus} · ${state.theme}`);
  result.push(pad(status, columns));
  result.push(
    pad(
      s.dim(
        " Tab: pane  ↑↓/jk: move  Enter: open  /: search  n/N: next  b: zen  t: theme  ?: help  q: quit",
      ),
      columns,
    ),
  );
  if (state.help) {
    const help = [
      "◆ READER KEYS",
      "",
      "↑ / k · ↓ / j     Scroll or select",
      "PgUp / PgDn       Scroll one page",
      "g / Home · G / End  Beginning / end",
      "Tab               Reader, files, outline, links",
      "Enter             Open file, heading or local link",
      "/ · n / N         Search, next / previous match",
      "b · t · r         Zen mode, theme, reload",
      "Esc · ?           Close help",
      "q · Ctrl-C        Quit and restore your terminal",
    ];
    const boxWidth = Math.min(columns - 4, 54),
      start = Math.max(2, Math.floor((rows - help.length) / 2));
    help.forEach((line, index) => {
      if (start + index < rows)
        result[start + index] = pad(
          " ".repeat(Math.floor((columns - boxWidth) / 2)) + s.selected(pad(` ${line}`, boxWidth)),
          columns,
        );
    });
  }
  return result.slice(0, rows).join("\r\n");
}

export function findMatch(lines, query, offset, direction = 1) {
  if (!query) return -1;
  for (let step = 1; step <= lines.length; step++) {
    const index = (offset + direction * step + lines.length) % lines.length;
    if (plain(lines[index]).toLocaleLowerCase().includes(query.toLocaleLowerCase())) return index;
  }
  return -1;
}
