import { emitKeypressEvents } from "node:readline";
import { watch } from "node:fs";
import { readFile, stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { styles, themeNames } from "./oxct-tui-style.mjs";
import { renderMarkdown } from "./oxct-tui-markdown.mjs";
import { findMatch, frame, layout } from "./oxct-tui-screen.mjs";

export async function readDocument(file) {
  if ((await stat(file)).size > 4 * 1024 * 1024)
    throw new Error("Document exceeds the 4 MiB viewer limit");
  return readFile(file, "utf8");
}

export async function startSession(
  files,
  root,
  options,
  io = { input: process.stdin, output: process.stdout },
) {
  const { input, output } = io;
  const state = {
    current: 0,
    fileIndex: 0,
    outlineIndex: 0,
    linkIndex: 0,
    offset: 0,
    focus: "reader",
    sidebar: true,
    theme: options.theme,
    query: "",
    searching: false,
    help: false,
    message: "",
  };
  let source = await readDocument(files[0]),
    document,
    size,
    watcher,
    timer,
    closed = false,
    loading = 0;
  const originalRaw = Boolean(input.isRaw);
  const draw = () => {
    if (closed) return;
    size = layout(output.columns, output.rows, state.sidebar);
    const s = styles(state.theme, options.color);
    document = renderMarkdown(source, size.contentWidth, s);
    state.offset = Math.max(
      0,
      Math.min(state.offset, Math.max(0, document.lines.length - size.height)),
    );
    output.write("\x1b[H" + frame(state, document, files, root, size, s) + "\x1b[J");
  };
  const monitor = () => {
    watcher?.close();
    if (options.watch) {
      // Watching the parent survives editor atomic-save renames.
      watcher = watch(dirname(files[state.current]), (_, filename) => {
        if (
          filename &&
          resolve(dirname(files[state.current]), filename.toString()) !== files[state.current]
        )
          return;
        clearTimeout(timer);
        timer = setTimeout(() => open(state.current, false), 100);
      });
      watcher.on("error", (error) => {
        state.message = error.message;
        draw();
      });
    }
  };
  const open = async (index, reset = true) => {
    const ticket = ++loading;
    try {
      const next = await readDocument(files[index]);
      if (closed || ticket !== loading) return;
      source = next;
      state.current = index;
      state.fileIndex = index;
      if (reset) {
        state.offset = 0;
        state.outlineIndex = 0;
        state.linkIndex = 0;
      }
      state.message = "";
      monitor();
      draw();
    } catch (error) {
      if (!closed) {
        state.message = error.message;
        draw();
      }
    }
  };
  let finish, fail;
  const completed = new Promise((resolve, reject) => {
    finish = resolve;
    fail = reject;
  });
  const close = (error) => {
    if (closed) return;
    closed = true;
    clearTimeout(timer);
    watcher?.close();
    input.off("keypress", handleKey);
    input.off("end", close);
    output.off("resize", draw);
    input.setRawMode(originalRaw);
    input.pause();
    process.off("SIGINT", close);
    process.off("SIGTERM", close);
    process.off("exit", restore);
    restore();
    if (error instanceof Error) fail(error);
    else finish();
  };
  const restore = () => {
    input.setRawMode(originalRaw);
    output.write("\x1b[0m\x1b[?25h\x1b[?1049l");
  };
  const search = (direction) => {
    const next = findMatch(document.lines, state.query, state.offset, direction);
    if (next === -1) state.message = "No matching text";
    else {
      state.offset = next;
      state.message = "";
    }
  };
  const onKey = (text, key = {}) => {
    // Escape followed quickly by q is emitted as Alt-q by terminal key parsers.
    if (key.meta && key.name === "q") {
      close();
      return;
    }
    if (key.ctrl && key.name === "c") {
      close();
      return;
    }
    if (state.searching) {
      if (key.name === "escape") {
        state.searching = false;
        state.query = "";
      } else if (key.name === "return") {
        state.searching = false;
        search(1);
      } else if (key.name === "backspace")
        state.query = Array.from(state.query).slice(0, -1).join("");
      else if (text && !key.ctrl && !key.meta)
        state.query += text.replace(/[\x00-\x1f\x7f-\x9f]/g, "");
      draw();
      return;
    }
    if (state.help) {
      if (["escape", "return"].includes(key.name) || text === "?") state.help = false;
      else if (text === "q") close();
      draw();
      return;
    }
    state.message = "";
    if (text === "q") {
      close();
      return;
    }
    if (text === "?") state.help = true;
    else if (text === "/") {
      state.searching = true;
      state.query = "";
    } else if (text === "n" || text === "N") search(text === "n" ? 1 : -1);
    else if (text === "b") state.sidebar = !state.sidebar;
    else if (text === "t")
      state.theme = themeNames[(themeNames.indexOf(state.theme) + 1) % themeNames.length];
    else if (text === "r") void open(state.current, false);
    else if (key.name === "tab") {
      const panes = ["reader", "files", "outline", "links"];
      state.focus = panes[(panes.indexOf(state.focus) + (key.shift ? 3 : 1)) % 4];
      state.sidebar = true;
    } else if (key.name === "escape") state.focus = "reader";
    else if (key.name === "return") {
      if (state.focus === "files") {
        void open(state.fileIndex);
        state.focus = "reader";
      } else if (state.focus === "outline") {
        state.offset = document.outline[state.outlineIndex]?.line ?? 0;
        state.focus = "reader";
      } else if (state.focus === "links") {
        const href = document.links[state.linkIndex]?.href;
        if (href && !/^[a-z][a-z\d+.-]*:/i.test(href) && !href.startsWith("//")) {
          try {
            const target = resolve(
              dirname(files[state.current]),
              decodeURIComponent(href.split(/[?#]/)[0]),
            );
            const index = files.indexOf(target);
            if (index >= 0) {
              void open(index);
              state.focus = "reader";
            } else state.message = "Local link is outside the selected Markdown files";
          } catch {
            state.message = "Invalid local link";
          }
        } else state.message = "External links are displayed as text";
      }
    } else {
      const direction =
        key.name === "up" || text === "k" ? -1 : key.name === "down" || text === "j" ? 1 : 0;
      const end = key.name === "end" || text === "G",
        home = key.name === "home" || text === "g";
      if (state.focus === "reader") {
        if (end) state.offset = document.lines.length;
        else if (home) state.offset = 0;
        else
          state.offset +=
            key.name === "pageup"
              ? -size.height
              : key.name === "pagedown" || text === " "
                ? size.height
                : direction;
      } else {
        const field =
          state.focus === "files"
            ? "fileIndex"
            : state.focus === "outline"
              ? "outlineIndex"
              : "linkIndex";
        const count =
          state.focus === "files"
            ? files.length
            : state.focus === "outline"
              ? document.outline.length
              : document.links.length;
        state[field] = Math.max(
          0,
          Math.min(Math.max(0, count - 1), end ? count - 1 : home ? 0 : state[field] + direction),
        );
      }
    }
    draw();
  };
  const handleKey = (text, key) => {
    try {
      onKey(text, key);
    } catch (error) {
      close(error);
    }
  };
  try {
    emitKeypressEvents(input);
    input.setRawMode(true);
    input.resume();
    input.on("keypress", handleKey);
    input.on("end", close);
    output.on("resize", draw);
    process.on("SIGINT", close);
    process.on("SIGTERM", close);
    process.on("exit", restore);
    output.write("\x1b[?1049h\x1b[?25l\x1b[2J");
    monitor();
    draw();
    await completed;
  } finally {
    close();
  }
}
