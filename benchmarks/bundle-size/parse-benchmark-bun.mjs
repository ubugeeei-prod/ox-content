import { performance } from "node:perf_hooks";

if (!globalThis.Bun?.markdown) {
  throw new Error("Bun.markdown is not available in this runtime");
}

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

function benchmark(name, fn, input, iterations = 100) {
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
    name,
    opsPerSec,
    avgMs,
    throughputMBs: (input.length / 1024 / 1024) * opsPerSec,
  };
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
  );
}

console.log(JSON.stringify({ render }));
