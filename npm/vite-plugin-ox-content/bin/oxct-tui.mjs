import { stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { glob } from "glob";
import { renderMarkdown } from "./oxct-tui-markdown.mjs";
import { styles, themeNames } from "./oxct-tui-style.mjs";
import { readDocument, startSession } from "./oxct-tui-session.mjs";

export async function runTui(args) {
  if (args.includes("--help") || args.includes("-h")) {
    console.log(`oxct tui [file/directory/glob]

A terminal Markdown reader with files, outline, local links, search, and live reload.

Options:
  --theme nord|light|mono  Color theme (default: nord)
  --print                 Render to stdout instead of opening a full-screen reader
  --stdin                 Render piped Markdown (print mode)
  --width <columns>       Print width, 20–240 (default: terminal width or 80)
  --no-watch              Disable live reload
  --no-color              Disable styling colors
  -h, --help              Show this help

Keys: Tab switches panes; j/k or arrows move; Enter opens; / searches; n/N finds
matches; b toggles zen mode; t changes theme; ? shows help; q or Ctrl-C quits.
Non-interactive output automatically uses print mode.`);
    return;
  }
  const options = parseTuiOptions(args);
  if (options.stdin) {
    const chunks = [];
    let bytes = 0;
    for await (const chunk of process.stdin) {
      bytes += Buffer.byteLength(chunk);
      if (bytes > 4 * 1024 * 1024) throw new Error("Document exceeds the 4 MiB viewer limit");
      chunks.push(Buffer.from(chunk));
    }
    print(Buffer.concat(chunks).toString("utf8"), options);
    return;
  }
  const target = resolve(options.path ?? ".");
  const info = await stat(target).catch(() => null);
  const root = info?.isDirectory() ? target : info?.isFile() ? dirname(target) : process.cwd();
  const files = info?.isFile()
    ? [target]
    : await glob(info?.isDirectory() ? "**/*.{md,markdown,mdx,mdc}" : options.path, {
        cwd: root,
        absolute: true,
        nodir: true,
        ignore: ["**/node_modules/**", "**/.git/**", "**/dist/**", "**/target/**"],
      });
  files.sort((a, b) => a.localeCompare(b));
  if (!files.length) throw new Error("No Markdown files matched the viewer input");
  if (files.length > 5000)
    throw new Error("Select a narrower directory or glob (viewer limit: 5000 files)");
  if (options.print || !process.stdin.isTTY || !process.stdout.isTTY)
    print(await readDocument(files[0]), options);
  else await startSession(files, root, options);
}

function print(source, options) {
  const document = renderMarkdown(source, options.width, styles(options.theme, options.color));
  process.stdout.write(document.lines.join("\n").trimEnd() + "\n");
}

export function parseTuiOptions(args) {
  const options = {
    theme: "nord",
    width: process.stdout.columns || 80,
    color: Boolean(process.stdout.isTTY) && !process.env.NO_COLOR,
    watch: true,
    print: false,
    stdin: false,
  };
  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (["--theme", "--width"].includes(arg)) {
      const value = args[++index];
      if (!value || value.startsWith("-")) throw new Error(`${arg} requires a value`);
      if (arg === "--theme") options.theme = value;
      else options.width = Number(value);
    } else if (arg === "--print") options.print = true;
    else if (arg === "--stdin") options.stdin = true;
    else if (arg === "--no-watch") options.watch = false;
    else if (arg === "--no-color") options.color = false;
    else if (arg.startsWith("-")) throw new Error(`Unknown tui option: ${arg}`);
    else if (options.path !== undefined) throw new Error(`Unexpected argument: ${arg}`);
    else options.path = arg;
  }
  if (!themeNames.includes(options.theme))
    throw new Error(`Unknown theme: ${options.theme}. Choose ${themeNames.join(", ")}.`);
  if (!Number.isInteger(options.width) || options.width < 20 || options.width > 240)
    throw new Error("--width must be an integer between 20 and 240");
  if (options.stdin && options.path) throw new Error("--stdin cannot be combined with a file path");
  return options;
}
