import { readFile } from "node:fs/promises";
import { relative, resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { loadNapi } from "./oxct-napi.mjs";

const defaultIgnore = ["**/node_modules/**", "**/.git/**", "**/dist/**", "**/target/**"];
const booleanRules = [
  "duplicateHeadings",
  "headingIncrement",
  "repeatedPunctuation",
  "repeatedWords",
  "spellcheck",
  "trailingSpaces",
];

export async function runLint(args) {
  if (args.includes("--help") || args.includes("-h")) {
    console.log(`oxct lint [files/globs]

Lint Markdown and MDX with the batched Rust engine. With no paths, search the current directory.

Options:
  --config <file>          JSON configuration: include, ignore, rules, languages, dictionary
  --ignore <glob>          Additional ignore pattern (repeatable)
  --format text|json       Output format (default: text)
  --stdin                  Read one document from stdin
  --stdin-filepath <path>  Filename used for stdin diagnostics and MDX detection
  --spellcheck             Enable the native built-in spelling dictionary
  --max-warnings <number>  Allowed warnings (default: 0)
  --no-color               Disable terminal colors
  -h, --help               Show this help`);
    return;
  }
  const started = performance.now();
  const options = await parseLintOptions(args);
  const napi = loadNapi();
  const files = options.stdin
    ? [{ path: options.stdinFilepath, source: await readStdin() }]
    : await discoverFiles(options);
  if (!files.length)
    throw new Error("No Markdown files matched. Check the paths and ignore patterns.");

  const diagnostics = [];
  let errorCount = 0;
  let warningCount = 0;
  // Bound file I/O and native allocations for repositories with large documents.
  for (let offset = 0; offset < files.length; offset += 128) {
    const batch = files.slice(offset, offset + 128);
    const sources = await Promise.all(
      batch.map((file) => file.source ?? readFile(file.path, "utf8")),
    );
    for (const mdx of [false, true]) {
      const indices = batch.flatMap((file, index) =>
        /\.mdx$/i.test(file.path) === mdx ? [index] : [],
      );
      if (!indices.length) continue;
      const results = napi.lintMarkdownDocuments(
        indices.map((index) => sources[index]),
        { ...options.native, mdx },
      );
      for (let index = 0; index < results.length; index++) {
        const result = results[index];
        errorCount += result.errorCount;
        warningCount += result.warningCount;
        const file = batch[indices[index]];
        diagnostics.push(
          ...result.diagnostics.map((diagnostic) => ({
            file: options.stdin
              ? file.path
              : relative(process.cwd(), file.path).replaceAll("\\", "/"),
            ...diagnostic,
          })),
        );
      }
    }
  }
  diagnostics.sort(
    (a, b) =>
      a.file.localeCompare(b.file) ||
      a.line - b.line ||
      a.column - b.column ||
      a.ruleId.localeCompare(b.ruleId),
  );
  const result = {
    checkedFileCount: files.length,
    errorCount,
    warningCount,
    diagnostics,
    durationMs: Number((performance.now() - started).toFixed(2)),
  };
  if (options.format === "json") console.log(JSON.stringify(result, null, 2));
  else emitText(result, options.color);
  if (errorCount > 0 || warningCount > options.maxWarnings) process.exitCode = 1;
}

export async function parseLintOptions(args) {
  const options = {
    paths: [],
    ignore: [],
    format: "text",
    stdin: false,
    stdinFilepath: "stdin.md",
    maxWarnings: 0,
    color: process.stdout.isTTY && !process.env.NO_COLOR,
    native: { rules: { spellcheck: false } },
  };
  let config;
  let spellcheck = false;
  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (["--config", "--ignore", "--format", "--stdin-filepath", "--max-warnings"].includes(arg)) {
      const value = args[++index];
      if (!value || value.startsWith("-")) throw new Error(`${arg} requires a value`);
      if (arg === "--config") config = value;
      if (arg === "--ignore") options.ignore.push(value);
      if (arg === "--format") options.format = value;
      if (arg === "--stdin-filepath") options.stdinFilepath = value;
      if (arg === "--max-warnings") options.maxWarnings = Number(value);
    } else if (arg === "--stdin") options.stdin = true;
    else if (arg === "--spellcheck") spellcheck = true;
    else if (arg === "--no-color") options.color = false;
    else if (arg.startsWith("-")) throw new Error(`Unknown lint option: ${arg}`);
    else options.paths.push(arg);
  }
  if (!["text", "json"].includes(options.format)) throw new Error("--format must be text or json");
  if (!Number.isSafeInteger(options.maxWarnings) || options.maxWarnings < 0)
    throw new Error("--max-warnings must be a nonnegative integer");
  if (options.stdin && options.paths.length)
    throw new Error("--stdin cannot be combined with file paths");
  if (config) {
    const data = JSON.parse(await readFile(resolve(config), "utf8"));
    validateConfig(data);
    if (!options.paths.length) options.paths = data.include ?? [];
    options.ignore.push(...(data.ignore ?? []));
    options.native = {
      languages: data.languages,
      dictionary: data.dictionary,
      rules: { spellcheck: false, ...data.rules },
    };
  }
  if (spellcheck) options.native.rules.spellcheck = true;
  return options;
}

function validateConfig(config) {
  if (!config || typeof config !== "object" || Array.isArray(config))
    throw new Error("Lint configuration must be an object");
  for (const key of Object.keys(config))
    if (!["include", "ignore", "rules", "languages", "dictionary"].includes(key))
      throw new Error(`Unknown lint configuration key: ${key}`);
  for (const key of ["include", "ignore", "languages"]) {
    if (
      config[key] !== undefined &&
      (!Array.isArray(config[key]) || config[key].some((value) => typeof value !== "string"))
    )
      throw new Error(`${key} must be an array of strings`);
  }
  if (config.rules !== undefined) {
    if (!config.rules || typeof config.rules !== "object" || Array.isArray(config.rules))
      throw new Error("rules must be an object");
    for (const [key, value] of Object.entries(config.rules)) {
      if (key === "maxConsecutiveBlankLines") {
        if (!Number.isSafeInteger(value) || value < 0 || value > 4294967295)
          throw new Error(`${key} must be a nonnegative 32-bit integer`);
      } else if (!booleanRules.includes(key) || typeof value !== "boolean")
        throw new Error(`Invalid lint rule: ${key}`);
    }
  }
  if (config.dictionary !== undefined) {
    const dictionary = config.dictionary;
    if (!dictionary || typeof dictionary !== "object" || Array.isArray(dictionary))
      throw new Error("dictionary must be an object");
    for (const key of Object.keys(dictionary))
      if (!["words", "ignoredWords", "byLanguage"].includes(key))
        throw new Error(`Unknown dictionary option: ${key}`);
    for (const key of ["words", "ignoredWords"])
      if (
        dictionary[key] !== undefined &&
        (!Array.isArray(dictionary[key]) ||
          dictionary[key].some((word) => typeof word !== "string"))
      )
        throw new Error(`${key} must be an array of strings`);
    if (
      dictionary.byLanguage !== undefined &&
      (!Array.isArray(dictionary.byLanguage) ||
        dictionary.byLanguage.some(
          (entry) =>
            !entry ||
            typeof entry.language !== "string" ||
            !Array.isArray(entry.words) ||
            entry.words.some((word) => typeof word !== "string"),
        ))
    )
      throw new Error("dictionary.byLanguage must be an array of { language, words } entries");
  }
}

async function discoverFiles(options) {
  const { glob } = await import("glob");
  const files = await glob(options.paths.length ? options.paths : ["**/*.{md,markdown,mdx,mdc}"], {
    cwd: process.cwd(),
    absolute: true,
    nodir: true,
    dot: true,
    ignore: [...defaultIgnore, ...options.ignore],
  });
  return [...new Set(files)].sort().map((path) => ({ path }));
}

async function readStdin() {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(Buffer.from(chunk));
  return Buffer.concat(chunks).toString("utf8");
}

function emitText(result, color) {
  const paint = (code, text) => (color ? `\x1b[${code}m${text}\x1b[0m` : text);
  console.log(paint("1;36", "◆ Ox Content · Markdown lint"));
  for (const diagnostic of result.diagnostics) {
    console.log(
      `${diagnostic.file}:${diagnostic.line}:${diagnostic.column} ${paint(diagnostic.severity === "error" ? "31" : "33", diagnostic.severity)} ${diagnostic.message} (${diagnostic.ruleId})`,
    );
  }
  console.log(
    `\n${result.checkedFileCount} files · ${result.errorCount} errors · ${result.warningCount} warnings · ${result.durationMs}ms`,
  );
}
