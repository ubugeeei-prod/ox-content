import * as fs from "node:fs/promises";
import * as http from "node:http";
import * as path from "node:path";

function contentTypeFor(filePath: string): string {
  const ext = path.extname(filePath).toLowerCase();
  switch (ext) {
    case ".html":
      return "text/html";
    case ".js":
      return "application/javascript";
    case ".css":
      return "text/css";
    case ".png":
      return "image/png";
    case ".svg":
      return "image/svg+xml";
    case ".json":
      return "application/json";
    case ".jpg":
    case ".jpeg":
      return "image/jpeg";
    case ".webp":
      return "image/webp";
    case ".woff":
      return "font/woff";
    case ".woff2":
      return "font/woff2";
    case ".ttf":
      return "font/ttf";
    default:
      return "application/octet-stream";
  }
}

async function resolveServedPath(rootDir: string, pathname: string): Promise<string> {
  const decodedPath = decodeURIComponent(pathname);
  const candidate = path.resolve(rootDir, decodedPath.replace(/^\/+/, ""));

  if (!candidate.startsWith(rootDir)) {
    throw new Error(`Refusing to serve path outside root: ${pathname}`);
  }

  if (path.extname(candidate)) {
    return candidate;
  }

  return path.join(candidate, "index.html");
}

/**
 * Starts a temporary static server for print-shell based PDF generation.
 */
export async function startStaticServer(
  rootDir: string,
): Promise<{ origin: string; close: () => Promise<void> }> {
  const server = http.createServer(async (req, res) => {
    const pathname = (req.url ?? "/").split("?")[0] ?? "/";

    try {
      const filePath = await resolveServedPath(rootDir, pathname);
      const body = await fs.readFile(filePath);
      res.writeHead(200, { "Content-Type": contentTypeFor(filePath) });
      res.end(body);
    } catch {
      res.writeHead(404, { "Content-Type": "text/plain" });
      res.end("not found");
    }
  });

  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      server.off("error", reject);
      resolve();
    });
  });

  const address = server.address();
  if (!address || typeof address === "string") {
    throw new Error("Unable to determine local server address for slide PDF export.");
  }

  return {
    origin: `http://127.0.0.1:${address.port}`,
    close: async () => {
      await new Promise<void>((resolve, reject) => {
        server.close((error) => (error ? reject(error) : resolve()));
      });
    },
  };
}

/**
 * Converts an output file path into a URL path served by `startStaticServer`.
 */
export function relativeUrlPath(rootDir: string, filePath: string): string {
  return path.relative(rootDir, filePath).split(path.sep).join("/");
}
