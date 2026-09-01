import { loadNapi } from "./oxct-napi.mjs";

export function runLinkCheck(args, fallback) {
  if (isHelp(args)) {
    printLinkCheckHelp();
    return;
  }

  const parsed = parseLinkCheckOptions(args);
  const napi = tryLoadNapi();
  if (!napi?.checkLinks) {
    fallback(args);
    return;
  }

  const result = napi.checkLinks(parsed.files, parsed.options);
  emitReports(result.reports, parsed.format);
  if (result.errorCount > 0) {
    process.exitCode = 1;
  }
}

export function runMdcCheck(args, fallback) {
  if (isHelp(args)) {
    printMdcCheckHelp();
    return;
  }

  const parsed = parseMdcCheckOptions(args);
  const napi = tryLoadNapi();
  if (!napi?.checkMdc) {
    fallback(args);
    return;
  }

  const result = napi.checkMdc(parsed.files);
  emitReports(result.reports, parsed.format);
  if (result.errorCount > 0) {
    process.exitCode = 1;
  }
}

function parseLinkCheckOptions(args) {
  const options = { ignore: [] };
  const files = [];
  let format = "text";

  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--src-dir") {
      options.srcDir = readValue(args, ++index, arg);
      continue;
    }
    if (arg === "--public-dir") {
      options.publicDir = readValue(args, ++index, arg);
      continue;
    }
    if (arg === "--site-dir") {
      options.siteDir = readValue(args, ++index, arg);
      continue;
    }
    if (arg === "--base") {
      options.base = readValue(args, ++index, arg);
      continue;
    }
    if (arg === "--ignore") {
      options.ignore.push(readValue(args, ++index, arg));
      continue;
    }
    if (arg === "--format") {
      format = readFormat(args, ++index, arg);
      continue;
    }
    if (arg.startsWith("--")) {
      throw new Error(`Unknown link-check option: ${arg}`);
    }
    files.push(arg);
  }

  if (files.length === 0 && !options.siteDir) {
    throw new Error("link-check requires one or more files, or --site-dir");
  }

  return { files, options, format };
}

function parseMdcCheckOptions(args) {
  const files = [];
  let format = "text";

  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--format") {
      format = readFormat(args, ++index, arg);
      continue;
    }
    if (arg.startsWith("--")) {
      throw new Error(`Unknown mdc-check option: ${arg}`);
    }
    files.push(arg);
  }

  if (files.length === 0) {
    throw new Error("mdc-check requires one or more files");
  }

  return { files, format };
}

function emitReports(reports, format) {
  if (format === "json") {
    console.log(JSON.stringify(reports, null, 2));
    return;
  }
  for (const report of reports) {
    for (const diagnostic of report.diagnostics) {
      console.log(
        `${report.file}:${diagnostic.line}:${diagnostic.column}: ${diagnostic.severity}: ${diagnostic.message}`,
      );
    }
  }
}

function tryLoadNapi() {
  try {
    return loadNapi();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    if (message.startsWith("Failed to load @ox-content/napi")) {
      return undefined;
    }
    throw error;
  }
}

function readFormat(args, index, option) {
  const format = readValue(args, index, option);
  if (format !== "text" && format !== "json") {
    throw new Error(`${option} must be text or json`);
  }
  return format;
}

function readValue(args, index, option) {
  const value = args[index];
  if (!value || value.startsWith("--")) {
    throw new Error(`${option} requires a value`);
  }
  return value;
}

function isHelp(args) {
  return args.length === 0 || args[0] === "--help" || args[0] === "-h";
}

function printLinkCheckHelp() {
  console.log(`oxct link-check

Usage:
  oxct link-check [--src-dir docs] [--public-dir public] [--site-dir dist] [--base /] [--ignore PATTERN] [--format text|json] <files...>`);
}

function printMdcCheckHelp() {
  console.log(`oxct mdc-check

Usage:
  oxct mdc-check [--format text|json] <files...>`);
}
