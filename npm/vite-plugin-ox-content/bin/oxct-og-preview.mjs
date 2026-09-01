import { readFileSync, writeFileSync } from "node:fs";
import { isAbsolute, resolve } from "node:path";
import { loadNapi } from "./oxct-napi.mjs";

export function runOgPreview(args) {
  if (args.length === 0 || args[0] === "--help" || args[0] === "-h") {
    printOgPreviewHelp();
    return;
  }

  const options = parseOgPreviewOptions(args);
  const svg = loadNapi().generateOgImageSvg(options.data, options.config);
  if (options.out) {
    writeFileSync(resolvePath(options.out), svg);
    return;
  }
  console.log(svg);
}

export function printOgPreviewHelp() {
  console.log(`oxct og-preview

Usage:
  oxct og-preview --title <text> [--description <text>] [--site-name <name>] [--author <name>] [--out og.svg]
  oxct og-preview --data page.json [--config og.json] [--out og.svg]

Options:
  --width <px>                    Image width.
  --height <px>                   Image height.
  --background-color <hex>        Background color.
  --text-color <hex>              Text color.
  --title-font-size <px>          Title font size.
  --description-font-size <px>    Description font size.`);
}

function parseOgPreviewOptions(args) {
  const data = {};
  const config = {};
  let out = "";

  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--data") {
      Object.assign(data, readJsonObject(readValue(args, ++index, arg), arg));
      continue;
    }
    if (arg === "--config") {
      Object.assign(config, readJsonObject(readValue(args, ++index, arg), arg));
      continue;
    }
    if (arg === "--title") {
      data.title = readValue(args, ++index, arg);
      continue;
    }
    if (arg === "--description") {
      data.description = readValue(args, ++index, arg);
      continue;
    }
    if (arg === "--site-name") {
      data.siteName = readValue(args, ++index, arg);
      continue;
    }
    if (arg === "--author") {
      data.author = readValue(args, ++index, arg);
      continue;
    }
    if (arg === "--out") {
      out = readValue(args, ++index, arg);
      continue;
    }
    if (arg === "--width" || arg === "--height") {
      config[arg.slice(2)] = readPositiveInteger(readValue(args, ++index, arg), arg);
      continue;
    }
    if (arg === "--background-color") {
      config.backgroundColor = readValue(args, ++index, arg);
      continue;
    }
    if (arg === "--text-color") {
      config.textColor = readValue(args, ++index, arg);
      continue;
    }
    if (arg === "--title-font-size") {
      config.titleFontSize = readPositiveInteger(readValue(args, ++index, arg), arg);
      continue;
    }
    if (arg === "--description-font-size") {
      config.descriptionFontSize = readPositiveInteger(readValue(args, ++index, arg), arg);
      continue;
    }
    throw new Error(`Unknown og-preview option: ${arg}`);
  }

  validateOgData(data);
  validateOgConfig(config);
  return { data, config, out };
}

function validateOgData(data) {
  if (typeof data.title !== "string" || data.title.length === 0) {
    throw new Error("og-preview requires --title or a --data JSON file with a title string");
  }
  for (const key of ["description", "siteName", "author"]) {
    if (data[key] !== undefined && typeof data[key] !== "string") {
      throw new Error(`og-preview data.${key} must be a string`);
    }
  }
}

function validateOgConfig(config) {
  for (const key of ["backgroundColor", "textColor"]) {
    if (config[key] !== undefined && typeof config[key] !== "string") {
      throw new Error(`og-preview config.${key} must be a string`);
    }
  }
  for (const key of ["width", "height", "titleFontSize", "descriptionFontSize"]) {
    if (config[key] !== undefined && !isPositiveInteger(config[key])) {
      throw new Error(`og-preview config.${key} must be a positive integer`);
    }
  }
}

function readJsonObject(file, option) {
  const parsed = JSON.parse(readFileSync(resolvePath(file), "utf8"));
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error(`${option} must point to a JSON object`);
  }
  return parsed;
}

function readValue(args, index, option) {
  const value = args[index];
  if (!value || value.startsWith("--")) {
    throw new Error(`${option} requires a value`);
  }
  return value;
}

function readPositiveInteger(value, option) {
  const parsed = Number(value);
  if (!isPositiveInteger(parsed)) {
    throw new Error(`${option} must be a positive integer`);
  }
  return parsed;
}

function isPositiveInteger(value) {
  return Number.isInteger(value) && value > 0;
}

function resolvePath(value) {
  return isAbsolute(value) ? value : resolve(process.cwd(), value);
}
