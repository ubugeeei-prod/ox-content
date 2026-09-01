#!/usr/bin/env node

import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

try {
  main(process.argv.slice(2));
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}

function main(args) {
  if (args.length === 0 || args[0] === "--help" || args[0] === "-h") {
    printHelp();
    return;
  }

  const [command, ...rest] = args;
  if (command === "i18n") {
    runI18n(rest);
    return;
  }

  throw new Error(`Unknown command: ${command}`);
}

function runI18n(args) {
  if (args.length === 0 || args[0] === "--help" || args[0] === "-h") {
    printI18nHelp();
    return;
  }

  const [command, ...rest] = args;
  if (command === "check") {
    runI18nCheck(parseCheckOptions(rest));
    return;
  }
  if (command === "validate") {
    runI18nValidate(parseValidateOptions(rest));
    return;
  }

  throw new Error(`Unknown i18n command: ${command}`);
}

function parseCheckOptions(args) {
  const options = {
    dictDir: "content/i18n",
    srcDirs: [],
    functionNames: [],
    format: "text",
    defaultLocale: "en",
  };

  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--dict-dir" || arg === "--dict") {
      options.dictDir = readValue(args, ++index, arg);
      continue;
    }
    if (arg === "--src") {
      options.srcDirs.push(readValue(args, ++index, arg));
      continue;
    }
    if (arg === "--function" || arg === "--function-name") {
      options.functionNames.push(readValue(args, ++index, arg));
      continue;
    }
    if (arg === "--format") {
      const format = readValue(args, ++index, arg);
      if (format !== "text" && format !== "json") {
        throw new Error("--format must be text or json");
      }
      options.format = format;
      continue;
    }
    if (arg === "--default-locale") {
      options.defaultLocale = readValue(args, ++index, arg);
      continue;
    }
    throw new Error(`Unknown i18n check option: ${arg}`);
  }

  if (options.srcDirs.length === 0) {
    options.srcDirs.push("src");
  }
  if (options.functionNames.length === 0) {
    options.functionNames.push("t", "$t");
  }

  return options;
}

function parseValidateOptions(args) {
  const options = { message: "", ast: false };
  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--ast") {
      options.ast = true;
      continue;
    }
    if (arg.startsWith("--")) {
      throw new Error(`Unknown i18n validate option: ${arg}`);
    }
    if (options.message) {
      throw new Error("i18n validate accepts exactly one message");
    }
    options.message = arg;
  }
  if (!options.message) {
    throw new Error("i18n validate requires a message");
  }
  return options;
}

function runI18nCheck(options) {
  const result = loadNapi().checkI18nProject(
    options.dictDir,
    options.srcDirs,
    options.functionNames,
    options.defaultLocale,
  );

  if (options.format === "json") {
    console.log(JSON.stringify(result, null, 2));
  } else {
    for (const diagnostic of result.diagnostics) {
      console.log(formatDiagnostic(diagnostic));
    }
    console.log(`\n${result.errorCount} error(s), ${result.warningCount} warning(s)`);
  }

  if (result.errorCount > 0) {
    process.exitCode = 1;
  }
}

function runI18nValidate(options) {
  const result = loadNapi().validateMf2(options.message);
  if (result.valid) {
    console.log("Valid MF2 message.");
  } else {
    for (const error of result.errors) {
      console.log(`error: ${error}`);
    }
    process.exitCode = 1;
  }
  if (options.ast && result.astJson) {
    console.log(`\nAST:\n${JSON.stringify(JSON.parse(result.astJson), null, 2)}`);
  }
}

function formatDiagnostic(diagnostic) {
  const context = [diagnostic.locale, diagnostic.key].filter(Boolean).join(" ");
  const suffix = context ? ` (${context})` : "";
  return `${diagnostic.severity}: ${diagnostic.message}${suffix}`;
}

function readValue(args, index, option) {
  const value = args[index];
  if (!value || value.startsWith("--")) {
    throw new Error(`${option} requires a value`);
  }
  return value;
}

function loadNapi() {
  try {
    return require("@ox-content/napi");
  } catch (error) {
    throw new Error(
      `Failed to load @ox-content/napi for oxct. Build the native package with \`vp run build:napi\` when running from the repository.\n${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

function printHelp() {
  console.log(`oxct

Usage:
  oxct i18n <command> [options]

Commands:
  i18n check      Check source translation keys against dictionaries
  i18n validate   Validate one ICU MessageFormat 2 message`);
}

function printI18nHelp() {
  console.log(`oxct i18n

Usage:
  oxct i18n check [--dict-dir content/i18n] [--src src] [--format text|json]
  oxct i18n validate [--ast] <message>`);
}
