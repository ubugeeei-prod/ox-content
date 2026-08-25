#!/usr/bin/env node

import { readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const allowlistPath = join(root, "config/panic-allowlist.json");
const cratesRoot = join(root, "crates");
const writeAllowlist = process.argv.includes("--write-allowlist");
const dump = process.argv.includes("--dump");

const KIND_PATTERNS = [
  { kind: "unwrap_unchecked", regex: /\.unwrap_unchecked\s*\(/g },
  { kind: "unwrap_err", regex: /\.unwrap_err\s*\(/g },
  { kind: "unwrap", regex: /\.unwrap\s*\(/g },
  { kind: "expect", regex: /\.expect\s*\(\s*"/g },
  { kind: "panic", regex: /\bpanic!\s*\(/g },
  { kind: "unreachable", regex: /\bunreachable!\s*\(/g },
  { kind: "todo", regex: /\btodo!\s*\(/g },
  { kind: "unimplemented", regex: /\bunimplemented!\s*\(/g },
];

const SKIP_DIR_NAMES = new Set(["tests", "benches", "examples", "target"]);

const findings = scanCrates(cratesRoot);
const counts = countByFileKind(findings);

if (writeAllowlist) {
  writeFileSync(allowlistPath, `${JSON.stringify(buildAllowlist(counts), null, 2)}\n`);
  console.log(`Wrote ${counts.length} allowlist entries to ${relative(root, allowlistPath)}`);
  process.exit(0);
}

if (dump) {
  for (const finding of findings) {
    console.log(`${finding.file}:${finding.line}:${finding.kind}: ${finding.text}`);
  }
  console.log(`\n${findings.length} production hits in ${counts.length} file/kind groups`);
  process.exit(0);
}

const allowlist = JSON.parse(readFileSync(allowlistPath, "utf8"));
const allowed = new Map(
  (allowlist.entries ?? []).map((entry) => [key(entry.file, entry.kind), entry]),
);
const extras = [];
const stale = [];

for (const group of counts) {
  const allow = allowed.get(key(group.file, group.kind));
  if (!allow) {
    extras.push({ ...group, reason: "not in allowlist" });
    continue;
  }
  if (group.count > allow.count) {
    extras.push({
      ...group,
      reason: `count ${group.count} exceeds allowlist ${allow.count}`,
    });
  }
}

for (const [entryKey, entry] of allowed) {
  const actual = counts.find((group) => key(group.file, group.kind) === entryKey);
  if (!actual) {
    stale.push({ ...entry, reason: "no remaining production hits" });
  } else if (actual.count < entry.count) {
    stale.push({
      ...entry,
      reason: `allowlist count ${entry.count} is higher than actual ${actual.count}`,
    });
  }
}

if (extras.length > 0 || stale.length > 0) {
  if (extras.length > 0) {
    console.error(
      "New panic-prone production constructs (update the allowlist only after review):",
    );
    for (const extra of extras) {
      console.error(`  - ${extra.file} ${extra.kind} x${extra.count} (${extra.reason})`);
    }
  }
  if (stale.length > 0) {
    console.error("Stale panic-allowlist entries (lower the count or remove the row):");
    for (const entry of stale) {
      console.error(`  - ${entry.file} ${entry.kind} x${entry.count} (${entry.reason})`);
    }
  }
  process.exit(1);
}

console.log(
  `Panic-construct gate passed: ${findings.length} reviewed production hits, ${allowed.size} allowlist entries.`,
);

function scanCrates(dir) {
  const files = [];
  walk(dir, files);
  const hits = [];
  for (const file of files) {
    hits.push(...scanFile(file));
  }
  return hits;
}

function walk(dir, files) {
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    const stat = statSync(path);
    if (stat.isDirectory()) {
      if (!SKIP_DIR_NAMES.has(entry)) {
        walk(path, files);
      }
      continue;
    }
    if (entry.endsWith(".rs") && !/(^|_)tests\.rs$/.test(entry)) {
      files.push(path);
    }
  }
}

function scanFile(absPath) {
  const rel = relative(root, absPath).replaceAll("\\", "/");
  const text = readFileSync(absPath, "utf8");
  const lines = text.split(/\r?\n/);
  const hits = [];
  let testDepth = 0;
  let pendingTestMod = false;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    const trimmed = line.trim();

    if (testDepth === 0 && /#\[cfg\(test\)\]/.test(trimmed)) {
      pendingTestMod = true;
    }
    if (pendingTestMod && /\bmod\s+\w+/.test(trimmed)) {
      pendingTestMod = false;
      if (trimmed.includes("{")) {
        testDepth = 1;
        testDepth += braceDelta(trimmed);
        continue;
      }
      continue;
    }
    if (pendingTestMod && trimmed === "") {
      continue;
    }
    if (pendingTestMod && !trimmed.startsWith("#[")) {
      pendingTestMod = false;
    }
    if (testDepth > 0) {
      testDepth += braceDelta(line);
      continue;
    }

    if (trimmed.startsWith("//") || trimmed.startsWith("///") || trimmed.startsWith("//!")) {
      continue;
    }

    for (const { kind, regex } of KIND_PATTERNS) {
      regex.lastIndex = 0;
      if (!regex.test(line)) {
        continue;
      }
      if (kind === "unwrap" && /\.unwrap_(or|or_else|or_default)\s*\(/.test(line)) {
        continue;
      }
      hits.push({ file: rel, line: index + 1, kind, text: trimmed });
      break;
    }
  }

  return hits;
}

function braceDelta(line) {
  let delta = 0;
  for (const char of line) {
    if (char === "{") {
      delta += 1;
    } else if (char === "}") {
      delta -= 1;
    }
  }
  return delta;
}

function countByFileKind(items) {
  const map = new Map();
  for (const item of items) {
    const entryKey = key(item.file, item.kind);
    const current = map.get(entryKey) ?? { file: item.file, kind: item.kind, count: 0 };
    current.count += 1;
    map.set(entryKey, current);
  }
  return [...map.values()].sort((left, right) =>
    left.file === right.file
      ? left.kind.localeCompare(right.kind)
      : left.file.localeCompare(right.file),
  );
}

function buildAllowlist(groups) {
  return {
    version: 1,
    description:
      "Reviewed production panic-prone constructs. New hits fail CI. Lower counts when a site is removed.",
    entries: groups.map((group) => ({
      file: group.file,
      kind: group.kind,
      count: group.count,
      reason: "reviewed; not an input-triggered abort on the current public surfaces",
    })),
  };
}

function key(file, kind) {
  return `${file}\0${kind}`;
}
