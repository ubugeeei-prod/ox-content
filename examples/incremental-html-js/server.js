import { createReadStream } from "node:fs";
import { access } from "node:fs/promises";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import { extname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL(".", import.meta.url));
const port = Number(process.env.PORT ?? 4174);
const require = createRequire(import.meta.url);
const napi = loadNapi();
const { IncrementalMarkdownRenderer } = napi;

const contentTypes = new Map([
  [".html", "text/html; charset=utf-8"],
  [".css", "text/css; charset=utf-8"],
  [".js", "text/javascript; charset=utf-8"],
]);

const staticFiles = new Map([
  ["/", "index.html"],
  ["/style.css", "style.css"],
  ["/client.js", "client.js"],
]);
const incrementalDomBuildPath = join(
  root,
  "../../npm/vite-plugin-ox-content/dist/incremental-dom.mjs",
);

function loadNapi() {
  try {
    return require("@ox-content/napi");
  } catch {
    return require("../../crates/ox_content_napi");
  }
}

const sample = `# Incremental renderer

This example streams a larger Markdown document through the native Rust renderer. The browser keeps a stable committed region and a replaceable pending region, so partial Markdown can look useful before the final delimiter arrives.

## Why the preview is split

The renderer returns two pieces on every append:

- **deltaHtml**: committed HTML that can be appended once and left alone.
- **pendingHtml**: provisional HTML for the unstable tail. The client replaces this region on the next chunk.
- **html**: a full snapshot for simple integrations that prefer replacing the whole preview.

That shape keeps the UI smooth without requiring a framework.

## Streaming behavior

Markdown often arrives in awkward boundaries. A heading can arrive as \`# Incre\`, emphasis can arrive as \`**strong\`, and code fences may stay open for several chunks. The incremental renderer keeps those cases readable by rendering the pending tail with temporary inline completion.

### Example update loop

\`\`\`js
const renderer = new IncrementalMarkdownRenderer({ gfm: true });

for await (const chunk of stream) {
  const result = renderer.append(chunk, {
    renderPending: true,
    completeInline: true,
  });

  committed.insertAdjacentHTML("beforeend", result.deltaHtml);
  pending.innerHTML = result.pendingHtml;
}

const final = renderer.finish();
committed.insertAdjacentHTML("beforeend", final.deltaHtml);
pending.innerHTML = "";
\`\`\`

## State model

| Field | Meaning |
| --- | --- |
| \`committedBytes\` | Bytes that will not be re-rendered |
| \`pendingBytes\` | Bytes still held as the unstable tail |
| \`didCommit\` | Whether this append produced new stable HTML |

## Longer content

This section exists to make the fade-in easier to inspect. The document is intentionally larger than a tiny chat message, with headings, paragraphs, a table, task list items, and a code block. Each committed segment is appended to the preview with a short transition, while the pending segment is replaced in place.

When the stream reaches a blank-line boundary followed by a new top-level block, the previous block becomes stable. Loose list continuations and open code fences stay pending, which avoids committing too aggressively.

- [x] Partial headings render as headings.
- [x] Partial emphasis can render as emphasized text.
- [x] Stable HTML does not need to be touched again.
- [ ] Your app can decide whether to use deltas or the full snapshot.

## Final section

The same API can back chat messages, documentation previews, or any append-only Markdown surface. For a framework integration, keep committed and pending regions as separate DOM nodes and let your UI library own the rest of the page.
`;

const chunks = splitMarkdown(sample, [12, 18, 24, 32, 45, 64, 28, 52, 80, 36, 72]);

function delay(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

function splitMarkdown(source, sizes) {
  const chunks = [];
  let cursor = 0;
  let sizeIndex = 0;

  while (cursor < source.length) {
    const size = sizes[sizeIndex % sizes.length];
    chunks.push(source.slice(cursor, cursor + size));
    cursor += size;
    sizeIndex += 1;
  }

  return chunks;
}

function sendEvent(response, event, data) {
  response.write(`event: ${event}\n`);
  response.write(`data: ${JSON.stringify(data)}\n\n`);
}

async function streamMarkdown(request, response) {
  response.writeHead(200, {
    "cache-control": "no-cache",
    connection: "keep-alive",
    "content-type": "text/event-stream; charset=utf-8",
  });

  let closed = false;
  request.on("close", () => {
    closed = true;
  });

  const renderer = new IncrementalMarkdownRenderer({
    gfm: true,
    strikethrough: true,
    tables: true,
    taskLists: true,
  });

  for (const chunk of chunks) {
    if (closed) {
      return;
    }
    const result = renderer.append(chunk, {
      renderPending: true,
      completeInline: true,
    });
    sendEvent(response, "chunk", { chunk, result });
    await delay(220);
  }

  if (!closed) {
    const result = renderer.finish();
    sendEvent(response, "finish", { chunk: "", result });
    response.end();
  }
}

function serveStatic(pathname, response) {
  if (pathname === "/incremental-dom.js") {
    response.writeHead(200, { "content-type": "text/javascript; charset=utf-8" });
    createReadStream(incrementalDomBuildPath).pipe(response);
    return;
  }

  const filename = staticFiles.get(pathname);
  if (!filename) {
    response.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
    response.end("Not found");
    return;
  }

  const filePath = join(root, filename);
  const contentType = contentTypes.get(extname(filename)) ?? "application/octet-stream";
  response.writeHead(200, { "content-type": contentType });
  createReadStream(filePath).pipe(response);
}

createServer((request, response) => {
  const url = new URL(request.url ?? "/", `http://${request.headers.host ?? "localhost"}`);
  if (url.pathname === "/stream") {
    void streamMarkdown(request, response);
    return;
  }
  if (url.pathname === "/incremental-dom.js") {
    access(incrementalDomBuildPath)
      .then(() => serveStatic(url.pathname, response))
      .catch(() => {
        response.writeHead(500, { "content-type": "text/plain; charset=utf-8" });
        response.end(
          "Missing npm/vite-plugin-ox-content/dist/incremental-dom.mjs. Run `corepack pnpm --dir npm/vite-plugin-ox-content build` first.",
        );
      });
    return;
  }
  serveStatic(url.pathname, response);
}).listen(port, () => {
  console.log(`Incremental HTML/JS example: http://localhost:${port}`);
});
