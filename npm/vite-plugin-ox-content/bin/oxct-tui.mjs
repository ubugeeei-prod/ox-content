import { runNativeCli } from "./oxct-native.mjs";

export function runTui(args) {
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
  runNativeCli("tui", args);
}
