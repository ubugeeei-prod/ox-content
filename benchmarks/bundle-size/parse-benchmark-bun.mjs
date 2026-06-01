import { performance } from "node:perf_hooks";

if (!globalThis.Bun?.markdown) {
  throw new Error("Bun.markdown is not available in this runtime");
}

const options = parseOptions(process.argv.slice(2));

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

const sizes = {
  small: sampleMarkdown,
  medium: Array(10).fill(sampleMarkdown).join("\n\n"),
  large: Array(100).fill(sampleMarkdown).join("\n\n"),
  // Keep the size set in sync with parse-benchmark.mjs (~1 MB document).
  huge: Array(2150).fill(sampleMarkdown).join("\n\n"),
};

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

function benchmarkOnce(fn, input, iterations) {
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

function medianSample(samples) {
  const sorted = [...samples].sort((a, b) => a.opsPerSec - b.opsPerSec);
  return sorted[Math.floor(sorted.length / 2)];
}

function parseOptions(args) {
  const parsed = {
    runs: 1,
  };

  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--runs") {
      parsed.runs = readPositiveIntegerOption(args, ++index, "--runs");
      continue;
    }
    if (arg.startsWith("--runs=")) {
      parsed.runs = parsePositiveInteger(arg.slice("--runs=".length), "--runs");
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

function readPositiveIntegerOption(args, index, optionName) {
  const value = args[index];
  if (!value || value.startsWith("--")) {
    throw new Error(`${optionName} requires a positive integer`);
  }

  return parsePositiveInteger(value, optionName);
}

function parsePositiveInteger(value, optionName) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isSafeInteger(parsed) || parsed < 1 || String(parsed) !== value) {
    throw new Error(`${optionName} requires a positive integer`);
  }

  return parsed;
}

function printUsage() {
  console.log(`Usage: bun parse-benchmark-bun.mjs [--runs <count>]

Options:
  --runs <count> Use the median result from repeated runs
  -h, --help     Show this help message`);
}

const render = {};

for (const [sizeName, content] of Object.entries(sizes)) {
  const iterations =
    sizeName === "huge" ? 5 : sizeName === "large" ? 20 : sizeName === "medium" ? 50 : 100;

  render[sizeName] = benchmark(
    "Bun.markdown.html",
    (input) => Bun.markdown.html(input),
    content,
    iterations,
    options.runs,
  );
}

console.log(JSON.stringify({ render }));
