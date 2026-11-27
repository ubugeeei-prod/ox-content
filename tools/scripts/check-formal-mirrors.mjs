#!/usr/bin/env node
// Checks that every function proven in `crates/*/formal` is the production
// function it stands for, once its proof is taken out.
//
// Creusot's own `#[erasure]` check would do this, but it does not support
// indexing (`bytes[i]`) yet, and the parser indexes everywhere. So the
// comparison is textual: drop the attributes, `ghost!`/`snapshot!`/
// `proof_assert!` statements and ghost arguments from the proven body, then
// compare it line by line with the production body. Comments and blank
// lines are ignored on both sides.

import { readFileSync } from "node:fs";

const mirrors = [
  {
    formal: "crates/ox_content_parser/formal/src/braces.rs",
    production: "crates/ox_content_parser/src/parser/mdx_jsx/braces.rs",
    functions: ["skip_quoted", "skip_line_comment", "skip_block_comment", "skip_braces"],
  },
  {
    formal: "crates/ox_content_parser/formal/src/lex_memo.rs",
    production: "crates/ox_content_parser/src/parser/mdx_jsx/lex_memo.rs",
    functions: ["backticks_end", "backtick_run_end"],
  },
];

/**
 * Length of the literal or comment at `index`, which brace counting must
 * not look into: `'{'`, `b'}'`, `"…"` and `// …`. Zero when there is none
 * (a lifetime such as `'a` is not a literal).
 */
function skipLength(source, index) {
  const rest = source.slice(index, index + 5);
  const char = /^b?'(\\.|[^\\'])'/.exec(rest);
  if (char) return char[0].length;
  if (source.startsWith("//", index)) {
    const end = source.indexOf("\n", index);
    return (end === -1 ? source.length : end) - index;
  }
  if (source[index] === '"') {
    let end = index + 1;
    while (end < source.length && source[end] !== '"') end += source[end] === "\\" ? 2 : 1;
    return end + 1 - index;
  }
  return 0;
}

/** The text between the braces of `fn name(...) ... { ... }`. */
function functionBody(source, name, file) {
  const signature = new RegExp(String.raw`\bfn ${name}\s*[(<]`);
  const match = signature.exec(source);
  if (!match) {
    throw new Error(`${file}: no function \`${name}\``);
  }
  // The body opens at the first `{` outside the signature's brackets.
  let depth = 0;
  let index = match.index;
  for (; index < source.length; index += 1) {
    const char = source[index];
    if (source.startsWith("->", index)) index += 1;
    else if ("(<[".includes(char)) depth += 1;
    else if (")>]".includes(char)) depth -= 1;
    else if (char === "{" && depth === 0) break;
  }
  const open = index;
  depth = 0;
  while (index < source.length) {
    const skip = skipLength(source, index);
    if (skip > 0) {
      index += skip;
      continue;
    }
    if (source[index] === "{") depth += 1;
    else if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(open + 1, index);
    }
    index += 1;
  }
  throw new Error(`${file}: unterminated body of \`${name}\``);
}

/** Removes `#[...]` attributes, which may span lines. */
function stripAttributes(text) {
  let out = "";
  for (let index = 0; index < text.length; index += 1) {
    if (text.startsWith("#[", index)) {
      let depth = 0;
      for (; index < text.length; index += 1) {
        if (text[index] === "[") depth += 1;
        else if (text[index] === "]") {
          depth -= 1;
          if (depth === 0) break;
        }
      }
      continue;
    }
    out += text[index];
  }
  return out;
}

const ghostStatement = /^(ghost!|proof_assert!|let \w+ = snapshot!)/;

function normalize(body, { formal }) {
  let text = formal ? stripAttributes(body) : body;
  if (formal) {
    text = text.replaceAll(/,\s*ghost!\(&mut \*\*steps\)/g, "");
  }
  const lines = [];
  let skipping = 0;
  for (const raw of text.split("\n")) {
    const line = raw.trim();
    if (formal && (skipping > 0 || ghostStatement.test(line))) {
      // A ghost statement may span lines; skip to its closing `;`.
      skipping += (line.match(/[({]/g) ?? []).length - (line.match(/[)}]/g) ?? []).length;
      if (skipping <= 0) skipping = 0;
      continue;
    }
    if (line === "" || line.startsWith("//")) continue;
    lines.push(line);
  }
  return lines;
}

let failures = 0;
for (const { formal, production, functions } of mirrors) {
  const formalSource = readFileSync(formal, "utf8");
  const productionSource = readFileSync(production, "utf8");
  for (const name of functions) {
    const proven = normalize(functionBody(formalSource, name, formal), { formal: true });
    const shipped = normalize(functionBody(productionSource, name, production), {
      formal: false,
    });
    const length = Math.max(proven.length, shipped.length);
    for (let line = 0; line < length; line += 1) {
      if (proven[line] !== shipped[line]) {
        failures += 1;
        console.error(`\`${name}\` in ${formal} no longer matches ${production}:`);
        console.error(`  proven:     ${proven[line] ?? "(end of body)"}`);
        console.error(`  production: ${shipped[line] ?? "(end of body)"}`);
        break;
      }
    }
  }
}

if (failures > 0) {
  console.error(
    "\nUpdate the proof in crates/ox_content_parser/formal and rerun `cargo creusot` there.",
  );
  process.exit(1);
}
console.log("Every proven function matches its production body.");
