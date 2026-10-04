import { runNativeCli } from "./oxct-native.mjs";

export function runLint(args) {
  if (args.includes("--help") || args.includes("-h")) {
    console.log(`oxct lint [files/globs]

Lint Markdown and MDX with the batched Rust engine. With no paths, search the current directory.

Options:
  --config <file>          JSON config (auto-detects .oxlint.json / oxlint.json)
  --ignore <glob>          Additional ignore pattern (repeatable)
  --format text|json       Output format (default: text)
  --stdin                  Read one document from stdin
  --stdin-filepath <path>  Filename used for stdin diagnostics and MDX detection
  --strict                 Enable all structural/style rules
  --fix                    Apply safe fixes to files and lint again
  --threads <number>       Worker count (default: available CPUs)
  --spellcheck             Enable the native built-in spelling dictionary
  --max-warnings <number>  Allowed warnings (default: 0)
  --no-color               Disable terminal colors
  -h, --help               Show this help`);
    return;
  }
  runNativeCli("lint", args);
}
