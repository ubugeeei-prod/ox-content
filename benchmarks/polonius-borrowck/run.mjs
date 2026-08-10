#!/usr/bin/env node

/**
 * Polonius Alpha vs NLL borrow-checker build-time benchmark.
 *
 * Since nightly-2026-08-06 the Polonius Alpha borrow checker is the default on
 * nightly, with stabilization planned before the end of 2026. This measures
 * what that switch costs this workspace: one nightly toolchain checks every
 * crate twice, once with `-Zpolonius=off` (today's stable NLL checker) and once
 * with `-Zpolonius=next` (Polonius Alpha), so the flag is the only variable.
 *
 * Two phases, because they answer different questions:
 *
 *   - `clean`: full `cargo check --workspace` from an empty target directory,
 *     dependencies included. This is the number a developer feels. CPU time
 *     (user + sys) is the headline metric — wall clock on a laptop has a
 *     standard deviation several times larger than the effect being measured,
 *     so it can only show that the effect is small, not how large it is.
 *
 *   - `borrowck`: each workspace crate re-checked on its own with
 *     `-Ztime-passes`, reporting the `MIR_borrow_checking` pass alone. One
 *     target per crate (lib, or bins when there is no lib), so this is where a
 *     borrow-checker change has to show up for library and binary code, and
 *     isolating it keeps the signal from drowning in dependency compile time.
 *     Per-crate minimum across repetitions, which is robust to scheduler noise.
 *
 * Each config gets its own `CARGO_TARGET_DIR` so alternating `RUSTFLAGS` does
 * not invalidate dependency fingerprints, and both phases alternate config
 * order per repetition (ABBA) to cancel thermal drift. `CARGO_INCREMENTAL=0`
 * keeps cargo from replaying cached output instead of recompiling.
 *
 * Usage:
 *   node benchmarks/polonius-borrowck/run.mjs
 *   node benchmarks/polonius-borrowck/run.mjs --iterations 4 --json results.json
 */

import { spawnSync } from "node:child_process";
import { globSync, mkdtempSync, rmSync, utimesSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

const CONFIGS = [
  { name: "nll", rustflags: "-Zpolonius=off" },
  { name: "polonius", rustflags: "-Zpolonius=next" },
];

const args = process.argv.slice(2);
const argValue = (flag) => {
  const index = args.indexOf(flag);
  return index === -1 ? undefined : args[index + 1];
};
const iterations = Number(argValue("--iterations") ?? 4);
const toolchain = argValue("--toolchain") ?? "nightly";
const jsonPath = argValue("--json");

if (!Number.isSafeInteger(iterations) || iterations < 1) {
  console.error("--iterations must be a positive integer");
  process.exit(1);
}

const targetDirs = new Map(
  CONFIGS.map((config) => [config.name, mkdtempSync(join(tmpdir(), `polonius-${config.name}-`))]),
);

/**
 * `cargo check` under `/usr/bin/time -p`, which reports the CPU consumed by the
 * whole `cargo` + `rustc` process tree. Node's `spawnSync` exposes no rusage,
 * and summing rustc's self-reported passes would miss cargo itself, so the
 * external timer is what makes the CPU metric trustworthy.
 */
function cargo(cargoArgs, { rustflags, targetDir }) {
  const started = process.hrtime.bigint();
  const result = spawnSync("/usr/bin/time", ["-p", "cargo", `+${toolchain}`, ...cargoArgs], {
    cwd: ROOT,
    env: {
      ...process.env,
      RUSTFLAGS: rustflags,
      CARGO_TARGET_DIR: targetDir,
      CARGO_INCREMENTAL: "0",
    },
    encoding: "utf8",
    maxBuffer: 256 * 1024 * 1024,
  });
  const stderr = result.stderr ?? "";
  const seconds = (name) =>
    Number(new RegExp(`^${name}\\s+([0-9.]+)$`, "m").exec(stderr)?.[1] ?? 0);
  return {
    wall: Number(process.hrtime.bigint() - started) / 1e9,
    cpu: seconds("user") + seconds("sys"),
    stderr,
    status: result.status ?? -1,
  };
}

/**
 * Sums one `-Ztime-passes` pass in a run's output.
 *
 * Only ever called on a single-crate `cargo rustc` run, which is what makes the
 * number trustworthy: cargo replays the cached stderr of already-built units,
 * so if dependencies had been built with `-Ztime-passes` their timing lines
 * would reappear on every later run and be counted as the crate under test.
 * Passing the flag through `cargo rustc --` applies it to this crate alone, so
 * no dependency ever emits a timing line to begin with.
 */
function passTotal(stderr, pass) {
  let total = 0;
  let found = false;
  for (const line of stderr.split("\n")) {
    const timed = /^time:\s+([0-9.]+);.*\t(\S+)$/.exec(line.trimEnd());
    if (timed && timed[2] === pass) {
      found = true;
      total += Number(timed[1]);
    }
  }
  // A missing pass means the crate was replayed from cache or `-Ztime-passes`
  // changed shape; either way a silent 0 would be recorded as a real timing.
  if (!found) throw new Error(`no ${pass} timing in the run output`);
  return total;
}

const samples = [];
const borrowck = new Map(CONFIGS.map((config) => [config.name, new Map()]));

const rustcVersion = spawnSync("rustc", [`+${toolchain}`, "--version"], {
  encoding: "utf8",
}).stdout?.trim();
if (!rustcVersion) {
  console.error(`toolchain "${toolchain}" is missing (rustup toolchain install ${toolchain})`);
  process.exit(1);
}
console.log(`${rustcVersion}\n${iterations} iterations per config, ABBA order\n`);

try {
  for (let iteration = 1; iteration <= iterations; iteration += 1) {
    const order = iteration % 2 === 1 ? CONFIGS : [...CONFIGS].reverse();
    for (const config of order) {
      const targetDir = targetDirs.get(config.name);
      rmSync(targetDir, { recursive: true, force: true });
      const run = cargo(["check", "--workspace"], { rustflags: config.rustflags, targetDir });
      if (run.status !== 0) {
        console.error(
          run.stderr
            .split("\n")
            .filter((l) => l.startsWith("error"))
            .join("\n"),
        );
        throw new Error(`${config.name} clean check failed (exit ${run.status})`);
      }
      samples.push({ iteration, config: config.name, phase: "clean", ...pick(run) });
      console.log(
        `clean     ${config.name.padEnd(8)} #${iteration}  ${run.cpu.toFixed(1)}s CPU  ${run.wall.toFixed(1)}s wall`,
      );
    }
  }

  // Dependencies are already primed in each config's target dir by the loop
  // above (without `-Ztime-passes`, see passTotal), so each crate is measured
  // alone: touch its root, re-check just that crate, read its own pass timing.
  // `cargo rustc` refuses to forward flags to more than one target, so each
  // crate is pinned to a single one: its lib when it has one, its bins
  // otherwise (six crates here ship both a lib and a CLI binary).
  const crateRoots = globSync("crates/*/src/{lib,main}.rs", { cwd: ROOT })
    .map((path) => ({ crate: path.split("/")[1], file: join(ROOT, path), path }))
    .filter(({ crate, path }, _, all) => {
      const isLib = path.endsWith("lib.rs");
      return isLib || !all.some((other) => other.crate === crate && other.path.endsWith("lib.rs"));
    })
    .map((entry) => ({ ...entry, target: entry.path.endsWith("lib.rs") ? "--lib" : "--bins" }));
  for (let rep = 1; rep <= iterations; rep += 1) {
    const order = rep % 2 === 1 ? CONFIGS : [...CONFIGS].reverse();
    for (const config of order) {
      const best = borrowck.get(config.name);
      for (const { crate, file, target } of crateRoots) {
        const now = new Date();
        utimesSync(file, now, now);
        const run = cargo(
          ["rustc", "-p", crate, "--profile", "check", target, "--", "-Ztime-passes"],
          { rustflags: config.rustflags, targetDir: targetDirs.get(config.name) },
        );
        if (run.status !== 0) throw new Error(`${config.name} borrowck run failed for ${crate}`);
        const seconds = passTotal(run.stderr, "MIR_borrow_checking");
        best.set(crate, Math.min(best.get(crate) ?? Infinity, seconds));
      }
      console.log(
        `borrowck  ${config.name.padEnd(8)} #${rep}  ${sum(best).toFixed(2)}s (best-so-far)`,
      );
    }
  }
} finally {
  for (const dir of targetDirs.values()) rmSync(dir, { recursive: true, force: true });
}

function pick({ wall, cpu, status }) {
  return { wall, cpu, status };
}
function sum(map) {
  return [...map.values()].reduce((a, b) => a + b, 0);
}
function mean(values) {
  return values.reduce((a, b) => a + b, 0) / values.length;
}

console.log("");
const summary = {};
for (const metric of ["cpu", "wall"]) {
  const means = {};
  for (const config of CONFIGS) {
    const values = samples.filter((s) => s.config === config.name).map((s) => s[metric]);
    means[config.name] = mean(values);
    const sd = Math.sqrt(mean(values.map((v) => (v - means[config.name]) ** 2)));
    console.log(
      `clean ${metric.padEnd(4)}  ${config.name.padEnd(8)} mean ${means[config.name].toFixed(1)}s ± ${sd.toFixed(1)}s`,
    );
  }
  const delta = means.polonius - means.nll;
  summary[`clean_${metric}`] = { ...means, delta, percent: (delta / means.nll) * 100 };
  console.log(
    `clean ${metric.padEnd(4)}  delta    ${delta > 0 ? "+" : ""}${delta.toFixed(1)}s (${((delta / means.nll) * 100).toFixed(1)}%)\n`,
  );
}

const borrowckTotals = Object.fromEntries(
  CONFIGS.map((config) => [config.name, sum(borrowck.get(config.name))]),
);
const borrowckDelta = borrowckTotals.polonius - borrowckTotals.nll;
summary.borrowck = {
  ...borrowckTotals,
  delta: borrowckDelta,
  percent: (borrowckDelta / borrowckTotals.nll) * 100,
};
console.log(
  `MIR_borrow_checking (workspace crates, min per crate)  nll ${borrowckTotals.nll.toFixed(2)}s  ` +
    `polonius ${borrowckTotals.polonius.toFixed(2)}s  ` +
    `delta ${borrowckDelta > 0 ? "+" : ""}${borrowckDelta.toFixed(2)}s ` +
    `(${((borrowckDelta / borrowckTotals.nll) * 100).toFixed(1)}%)`,
);

if (jsonPath) {
  const perCrate = Object.fromEntries(
    CONFIGS.map((config) => [config.name, Object.fromEntries(borrowck.get(config.name))]),
  );
  writeFileSync(
    jsonPath,
    `${JSON.stringify({ rustcVersion, summary, samples, perCrate }, null, 2)}\n`,
  );
  console.log(`\nwrote ${jsonPath}`);
}
