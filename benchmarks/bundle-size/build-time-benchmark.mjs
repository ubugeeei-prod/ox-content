#!/usr/bin/env node

/**
 * Build Time Benchmark
 *
 * Measures production build time for various documentation frameworks.
 */

import { execSync } from "node:child_process";
import { existsSync, rmSync, writeFileSync } from "node:fs";
import { cpus, totalmem } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const APPS_DIR = join(dirname(fileURLToPath(import.meta.url)), "apps");
const ITERATIONS = 3;
const options = parseOptions(process.argv.slice(2));

const apps = [
  { name: "ox-content (bare)", dir: "ox-content-bare", buildCmd: "npm run build" },
  { name: "ox-content (default)", dir: "ox-content", buildCmd: "npm run build" },
  { name: "ox-content + Vue", dir: "ox-content-vue", buildCmd: "npm run build" },
  { name: "VitePress (bare)", dir: "vitepress-bare", buildCmd: "npm run build" },
  { name: "VitePress (default)", dir: "vitepress", buildCmd: "npm run build" },
  { name: "Astro", dir: "astro", buildCmd: "npm run build" },
  { name: "Astro + Vue", dir: "astro-vue", buildCmd: "npm run build" },
];

function parseOptions(args) {
  const parsed = { jsonPath: null };

  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--json") {
      parsed.jsonPath = readOptionValue(args, ++index, "--json");
      continue;
    }
    if (arg.startsWith("--json=")) {
      parsed.jsonPath = arg.slice("--json=".length);
      if (!parsed.jsonPath) {
        throw new Error("--json requires a file path");
      }
      continue;
    }
    if (arg === "--help" || arg === "-h") {
      console.log("Usage: node build-time-benchmark.mjs [--json <path>]");
      process.exit(0);
    }
    throw new Error(`Unknown argument: ${arg}`);
  }

  return parsed;
}

function readOptionValue(args, index, optionName) {
  const value = args[index];
  if (!value || value.startsWith("--")) {
    throw new Error(`${optionName} requires a file path`);
  }
  return value;
}

function cleanDist(appDir) {
  const distPaths = [join(appDir, "dist"), join(appDir, ".vitepress/dist"), join(appDir, ".astro")];
  for (const distPath of distPaths) {
    if (existsSync(distPath)) {
      rmSync(distPath, { recursive: true, force: true });
    }
  }
}

function measureBuildTime(appDir, buildCmd) {
  const start = performance.now();
  try {
    execSync(buildCmd, {
      cwd: appDir,
      stdio: "pipe",
      env: { ...process.env, NODE_ENV: "production" },
    });
    return performance.now() - start;
  } catch (error) {
    console.error(`Build failed in ${appDir}:`, error.message);
    return null;
  }
}

function collectEnvironment() {
  const cpuList = cpus();
  return {
    node: process.version,
    platform: process.platform,
    arch: process.arch,
    ci: process.env.CI === "true",
    runnerLabel: process.env.OX_CONTENT_BENCHMARK_RUNNER ?? null,
    cpuModel: cpuList[0]?.model ?? null,
    cpuCount: cpuList.length,
    totalMemoryGB: Number((totalmem() / 1024 ** 3).toFixed(2)),
  };
}

async function runBenchmark() {
  console.log("Build Time Benchmark");
  console.log("====================\n");
  console.log(`Running ${ITERATIONS} iterations per framework...\n`);

  const results = [];

  for (const app of apps) {
    const appDir = join(APPS_DIR, app.dir);

    if (!existsSync(appDir)) {
      console.log(`  ${app.name}: skipped (not found)`);
      continue;
    }

    process.stdout.write(`  ${app.name}: `);

    const times = [];
    for (let i = 0; i < ITERATIONS; i++) {
      cleanDist(appDir);

      const time = measureBuildTime(appDir, app.buildCmd);
      if (time !== null) {
        times.push(time);
        process.stdout.write(".");
      } else {
        process.stdout.write("x");
      }
    }

    if (times.length > 0) {
      const avg = times.reduce((a, b) => a + b, 0) / times.length;
      const min = Math.min(...times);
      const max = Math.max(...times);
      results.push({
        name: app.name,
        avg,
        min,
        max,
        times,
      });
      console.log(` ${avg.toFixed(0)}ms (avg)`);
    } else {
      console.log(" failed");
    }
  }

  results.sort((a, b) => a.avg - b.avg);

  const baseline = results.find((r) => r.name === "ox-content (bare)")?.avg || results[0]?.avg;

  console.log("\n\nResults (sorted by build time):");
  console.log("================================\n");

  const nameWidth = Math.max(9, ...results.map((r) => r.name.length));
  const avgWidth = 8;
  const minWidth = 8;
  const maxWidth = 8;
  const ratioWidth = 8;

  const header = `| ${"Framework".padEnd(nameWidth)} | ${"Avg".padStart(avgWidth)} | ${"Min".padStart(minWidth)} | ${"Max".padStart(maxWidth)} | ${"Ratio".padStart(ratioWidth)} |`;
  const separator = `|${"-".repeat(nameWidth + 2)}|${"-".repeat(avgWidth + 2)}|${"-".repeat(minWidth + 2)}|${"-".repeat(maxWidth + 2)}|${"-".repeat(ratioWidth + 2)}|`;

  console.log(header);
  console.log(separator);

  for (const result of results) {
    const ratio = result.avg / baseline;
    const avgStr = `${result.avg.toFixed(0)}ms`.padStart(avgWidth);
    const minStr = `${result.min.toFixed(0)}ms`.padStart(minWidth);
    const maxStr = `${result.max.toFixed(0)}ms`.padStart(maxWidth);
    const ratioStr = `${ratio.toFixed(2)}x`.padStart(ratioWidth);

    console.log(
      `| ${result.name.padEnd(nameWidth)} | ${avgStr} | ${minStr} | ${maxStr} | ${ratioStr} |`,
    );
  }

  console.log("\n");
  console.log("Notes:");
  console.log("- All frameworks built with production settings");
  console.log(`- Average of ${ITERATIONS} builds per framework`);
  console.log("- Disk cache cleared between builds");
  console.log("- Same markdown content used across all frameworks");

  if (options.jsonPath) {
    writeFileSync(
      options.jsonPath,
      `${JSON.stringify(
        {
          name: "Build Time Benchmark",
          generatedAt: new Date().toISOString(),
          iterations: ITERATIONS,
          environment: collectEnvironment(),
          baseline,
          results,
        },
        null,
        2,
      )}\n`,
    );
    console.log(`\nWrote build time JSON to ${options.jsonPath}`);
  }
}

runBenchmark().catch(console.error);
