/**
 * Parse/Render speed benchmark
 * Measures parse and render throughput for markdown libraries
 */

import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { performance } from "node:perf_hooks";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const BUN_BENCHMARK_SCRIPT = join(__dirname, "parse-benchmark-bun.mjs");
const options = parseOptions(process.argv.slice(2));

/**
 * @typedef {Object} BenchmarkOptions
 * @property {string | null} jsonPath
 * @property {number} runs
 */

// Sample markdown content for benchmarking
const sampleMarkdown = `
# Heading 1

This is a paragraph with **bold** and *italic* text.

## Heading 2

- List item 1
- List item 2
  - Nested item
- List item 3

### Code Block

\`\`\`javascript
function hello() {
  console.log("Hello, World!");
}
\`\`\`

> This is a blockquote
> with multiple lines

| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |

Here's a [link](https://example.com) and an image: ![alt](image.png)

---

Final paragraph with \`inline code\` and more text.
`;

// Repeat the sample to create larger documents
const sizes = {
  small: sampleMarkdown,
  medium: Array(10).fill(sampleMarkdown).join("\n\n"),
  large: Array(100).fill(sampleMarkdown).join("\n\n"),
};

/**
 * Benchmark a sync function
 */
function benchmark(name, fn, input, iterations = 100, runs = 1) {
  const samples = [];

  for (let run = 0; run < runs; run++) {
    samples.push(benchmarkOnce(fn, input, iterations));
  }

  return {
    name,
    ...medianSample(samples),
    samples,
  };
}

/**
 * @param {string[]} args
 * @returns {BenchmarkOptions}
 */
function parseOptions(args) {
  /** @type {BenchmarkOptions} */
  const parsed = {
    jsonPath: null,
    runs: 1,
  };

  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--json") {
      parsed.jsonPath = readOptionValue(args, ++index, "--json");
      continue;
    }
    if (arg.startsWith("--json=")) {
      parsed.jsonPath = readInlineOptionValue(arg, "--json");
      continue;
    }
    if (arg === "--runs") {
      parsed.runs = readPositiveIntegerOption(args, ++index, "--runs");
      continue;
    }
    if (arg.startsWith("--runs=")) {
      parsed.runs = readPositiveIntegerInlineOption(arg, "--runs");
      continue;
    }
    if (arg === "--help" || arg === "-h") {
      printUsage();
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return parsed;
}

function benchmarkOnce(fn, input, iterations) {
  // Warmup
  for (let i = 0; i < 5; i++) {
    fn(input);
  }

  const start = performance.now();
  for (let i = 0; i < iterations; i++) {
    fn(input);
  }
  const elapsed = performance.now() - start;

  const avgMs = elapsed / iterations;
  const opsPerSec = 1000 / avgMs;

  return {
    opsPerSec,
    avgMs,
    throughputMBs: (input.length / 1024 / 1024) * opsPerSec,
  };
}

/**
 * @param {string[]} args
 * @param {number} index
 * @param {string} optionName
 * @returns {string}
 */
function readOptionValue(args, index, optionName) {
  const value = args[index];
  if (!value || value.startsWith("--")) {
    throw new Error(`${optionName} requires a file path`);
  }

  return value;
}

/**
 * @param {string[]} args
 * @param {number} index
 * @param {string} optionName
 * @returns {number}
 */
function readPositiveIntegerOption(args, index, optionName) {
  return parsePositiveInteger(readOptionValue(args, index, optionName), optionName);
}

/**
 * @param {string} arg
 * @param {string} optionName
 * @returns {string}
 */
function readInlineOptionValue(arg, optionName) {
  const value = arg.slice(`${optionName}=`.length);
  if (!value) {
    throw new Error(`${optionName} requires a file path`);
  }

  return value;
}

/**
 * @param {string} arg
 * @param {string} optionName
 * @returns {number}
 */
function readPositiveIntegerInlineOption(arg, optionName) {
  return parsePositiveInteger(readInlineOptionValue(arg, optionName), optionName);
}

function parsePositiveInteger(value, optionName) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isSafeInteger(parsed) || parsed < 1 || String(parsed) !== value) {
    throw new Error(`${optionName} requires a positive integer`);
  }

  return parsed;
}

function medianSample(samples) {
  const sorted = [...samples].sort((a, b) => a.opsPerSec - b.opsPerSec);
  return sorted[Math.floor(sorted.length / 2)];
}

function printUsage() {
  console.log(`Usage: node parse-benchmark.mjs [--json <path>] [--runs <count>]

Options:
  --json <path>  Write benchmark results as JSON
  --runs <count> Use the median result from repeated runs
  -h, --help     Show this help message`);
}

/**
 * Benchmark an async function
 */
async function benchmarkAsync(name, fn, input, iterations = 100) {
  const samples = [];

  for (let run = 0; run < options.runs; run++) {
    samples.push(await benchmarkAsyncOnce(fn, input, iterations));
  }

  return {
    name,
    ...medianSample(samples),
    samples,
  };
}

async function benchmarkAsyncOnce(fn, input, iterations) {
  // Warmup
  for (let i = 0; i < 5; i++) {
    await fn(input);
  }

  const start = performance.now();
  for (let i = 0; i < iterations; i++) {
    await fn(input);
  }
  const elapsed = performance.now() - start;

  const avgMs = elapsed / iterations;
  const opsPerSec = 1000 / avgMs;

  return {
    opsPerSec,
    avgMs,
    throughputMBs: (input.length / 1024 / 1024) * opsPerSec,
  };
}

/**
 * Print benchmark results table
 */
function printTable(title, results) {
  console.log(`\n### ${title}\n`);

  // Calculate dynamic column widths
  const nameWidth = Math.max(7, ...results.map((r) => r.name.length)); // min 7 for "Library"
  const opsWidth = 10;
  const avgWidth = 9;
  const throughputWidth = 11;
  const ratioWidth = 7;

  // Print header
  const header = `| ${"Library".padEnd(nameWidth)} | ${"ops/sec".padStart(opsWidth)} | ${"avg time".padStart(avgWidth)} | ${"throughput".padStart(throughputWidth)} | ${"ratio".padStart(ratioWidth)} |`;
  const separator = `|${"-".repeat(nameWidth + 2)}|${"-".repeat(opsWidth + 2)}|${"-".repeat(avgWidth + 2)}|${"-".repeat(throughputWidth + 2)}|${"-".repeat(ratioWidth + 2)}|`;
  console.log(header);
  console.log(separator);

  // Find the fastest (highest ops/sec) for ratio calculation
  const validResults = results.filter((r) => !r.error && r.opsPerSec > 0);
  const fastest = Math.max(...validResults.map((r) => r.opsPerSec));

  for (const result of results) {
    if (result.error) {
      console.log(
        `| ${result.name.padEnd(nameWidth)} | ${"Error".padStart(opsWidth)} | ${"-".padStart(avgWidth)} | ${"-".padStart(throughputWidth)} | ${"-".padStart(ratioWidth)} |`,
      );
      continue;
    }
    const name = result.name.padEnd(nameWidth);
    const opsPerSec = result.opsPerSec.toFixed(0).padStart(opsWidth);
    const avgMs = (result.avgMs.toFixed(2) + "ms").padStart(avgWidth);
    const throughput = (result.throughputMBs.toFixed(2) + " MB/s").padStart(throughputWidth);
    const ratio = (fastest / result.opsPerSec).toFixed(2) + "x";
    console.log(
      `| ${name} | ${opsPerSec} | ${avgMs} | ${throughput} | ${ratio.padStart(ratioWidth)} |`,
    );
  }
}

function loadBunMarkdownBenchmarks() {
  const version = spawnSync("bun", ["--version"], {
    cwd: __dirname,
    encoding: "utf8",
  });

  if (version.status !== 0) {
    return null;
  }

  const run = spawnSync("bun", [BUN_BENCHMARK_SCRIPT], {
    cwd: __dirname,
    encoding: "utf8",
  });

  if (run.status !== 0) {
    const details = run.stderr.trim() || run.stdout.trim();
    console.warn(`Failed to run Bun benchmark helper: ${details}`);
    return null;
  }

  try {
    return {
      version: version.stdout.trim(),
      ...JSON.parse(run.stdout),
    };
  } catch (error) {
    console.warn(`Failed to parse Bun benchmark output: ${String(error)}`);
    return null;
  }
}

async function runBenchmarks() {
  console.log("Parse/Render Speed Benchmark");
  console.log("============================\n");
  if (options.runs > 1) {
    console.log(`Using median of ${options.runs} runs\n`);
  }

  const report = {
    name: "Parse/Render Speed Benchmark",
    generatedAt: new Date().toISOString(),
    runs: options.runs,
    sizes: {},
  };

  // Import libraries
  const { marked } = await import("marked");
  const { Lexer: MarkedLexer } = await import("marked");
  const MarkdownIt = (await import("markdown-it")).default;
  const { init: initMd4w, mdToHtml, mdToJSON } = await import("md4w");
  const { parseAST: md4xParseAST, renderToHtml: md4xRenderToHtml } = await import("md4x/napi");
  const { micromark } = await import("micromark");
  const { unified } = await import("unified");
  const remarkParse = (await import("remark-parse")).default;
  const remarkHtml = (await import("remark-html")).default;
  const { markdownToMdast: satteriParse, markdownToHtml: satteriRender } = await import("satteri");
  await initMd4w();

  // Try to import NAPI
  let napi = null;
  try {
    napi = await import("@ox-content/napi");
    console.log("Using @ox-content/napi (Native)\n");
  } catch {
    console.log("@ox-content/napi not available\n");
  }

  const bunMarkdown = loadBunMarkdownBenchmarks();
  if (bunMarkdown) {
    console.log(`Using Bun.markdown (Bun ${bunMarkdown.version})\n`);
  } else {
    console.log("bun not available, skipping Bun.markdown comparisons\n");
  }

  const md = new MarkdownIt();
  const remarkParseProcessor = unified().use(remarkParse);
  const remarkFullProcessor = unified().use(remarkParse).use(remarkHtml);

  // Define parsers (parse only)
  const parsers = [];

  if (napi) {
    parsers.push({
      name: "@ox-content/napi",
      fn: (input) => napi.parse(input),
    });
  }

  parsers.push(
    { name: "marked", fn: (input) => MarkedLexer.lex(input) },
    { name: "md4w (md4c)", fn: (input) => mdToJSON(input) },
    { name: "md4x (napi)", fn: (input) => md4xParseAST(input) },
    { name: "markdown-it", fn: (input) => md.parse(input, {}) },
    { name: "remark", fn: (input) => remarkParseProcessor.parse(input) },
    { name: "satteri", fn: (input) => satteriParse(input) },
  );

  // Define renderers (parse + render)
  const renderers = [];

  if (napi) {
    renderers.push({
      name: "@ox-content/napi",
      fn: (input) => napi.parseAndRender(input).html,
    });
  }

  renderers.push(
    { name: "marked", fn: (input) => marked(input) },
    { name: "md4w (md4c)", fn: (input) => mdToHtml(input) },
    { name: "md4x (napi)", fn: (input) => md4xRenderToHtml(input) },
    { name: "markdown-it", fn: (input) => md.render(input) },
    { name: "micromark", fn: (input) => micromark(input) },
    {
      name: "remark",
      fn: (input) => remarkFullProcessor.processSync(input).toString(),
    },
    { name: "satteri", fn: (input) => satteriRender(input) },
  );

  // Define async renderers
  const asyncRenderers = [];

  if (napi?.parseAndRenderAsync) {
    asyncRenderers.push({
      name: "@ox-content/napi (async)",
      fn: (input) => napi.parseAndRenderAsync(input),
    });
  }

  for (const [sizeName, content] of Object.entries(sizes)) {
    const sizeKB = (content.length / 1024).toFixed(1);
    console.log(`\n## ${sizeName.toUpperCase()} (${sizeKB} KB)`);

    const iterations = sizeName === "large" ? 20 : sizeName === "medium" ? 50 : 100;
    const suites = {};

    // Parse only benchmark
    const parseResults = [];
    for (const parser of parsers) {
      try {
        const result = benchmark(parser.name, parser.fn, content, iterations, options.runs);
        parseResults.push(result);
      } catch {
        parseResults.push({ name: parser.name, error: true });
      }
    }
    parseResults.sort((a, b) => (b.opsPerSec || 0) - (a.opsPerSec || 0));
    suites.parseOnly = parseResults;
    printTable("Parse Only", parseResults);

    // Parse + Render benchmark
    const renderResults = [];
    for (const renderer of renderers) {
      try {
        const result = benchmark(renderer.name, renderer.fn, content, iterations, options.runs);
        renderResults.push(result);
      } catch {
        renderResults.push({ name: renderer.name, error: true });
      }
    }
    if (bunMarkdown?.render?.[sizeName]) {
      renderResults.push(bunMarkdown.render[sizeName]);
    }
    renderResults.sort((a, b) => (b.opsPerSec || 0) - (a.opsPerSec || 0));
    suites.parseAndRender = renderResults;
    printTable("Parse + Render", renderResults);

    // Async benchmark (only for large)
    if (asyncRenderers.length > 0 && sizeName === "large") {
      const asyncResults = [];
      for (const renderer of asyncRenderers) {
        try {
          const result = await benchmarkAsync(renderer.name, renderer.fn, content, iterations);
          asyncResults.push(result);
        } catch {
          asyncResults.push({ name: renderer.name, error: true });
        }
      }
      asyncResults.sort((a, b) => (b.opsPerSec || 0) - (a.opsPerSec || 0));
      suites.parseAndRenderAsync = asyncResults;
      printTable("Parse + Render (Async/Worker Thread)", asyncResults);
    }

    report.sizes[sizeName] = {
      bytes: content.length,
      sizeKB: Number(sizeKB),
      iterations,
      runs: options.runs,
      suites,
    };
  }

  console.log("\n\n*Higher ops/sec and throughput = better.*");

  return report;
}

runBenchmarks()
  .then((report) => {
    if (options.jsonPath) {
      writeFileSync(options.jsonPath, `${JSON.stringify(report, null, 2)}\n`);
      console.log(`\nWrote benchmark JSON to ${options.jsonPath}`);
    }
  })
  .catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
