/**
 * OG Viewer - Dev tool for previewing Open Graph metadata
 *
 * Accessible at /__og-viewer during development.
 * Shows all pages with their OG metadata, validation warnings,
 * and social card previews.
 */

import * as fs from "fs";
import * as path from "path";
import { glob } from "glob";
import type { Plugin } from "vite";
import type { ResolvedOptions } from "./types";

// =============================================================================
// Types
// =============================================================================

interface PageOgData {
  path: string;
  urlPath: string;
  title: string;
  description: string;
  author: string;
  tags: string[];
  ogImageUrl: string;
  warnings: { level: "error" | "warning"; message: string }[];
}

// =============================================================================
// Data Collection
// =============================================================================

function parseFrontmatter(content: string): Record<string, unknown> {
  const match = content.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!match) return {};

  const yaml = match[1];
  const result: Record<string, unknown> = {};

  for (const line of yaml.split("\n")) {
    const kv = line.match(/^(\w[\w-]*):\s*(.*)$/);
    if (!kv) continue;
    const [, key, rawValue] = kv;
    let value: unknown = rawValue.trim();

    // Handle arrays (simple inline: [a, b])
    if (typeof value === "string" && value.startsWith("[") && value.endsWith("]")) {
      value = value
        .slice(1, -1)
        .split(",")
        .map((s) => s.trim().replace(/^['"]|['"]$/g, ""))
        .filter(Boolean);
    }
    // Strip quotes
    else if (typeof value === "string" && /^['"].*['"]$/.test(value)) {
      value = value.slice(1, -1);
    }
    // Booleans
    else if (value === "true") value = true;
    else if (value === "false") value = false;

    result[key] = value;
  }

  return result;
}

function extractTitle(content: string, frontmatter: Record<string, unknown>): string {
  if (typeof frontmatter.title === "string" && frontmatter.title) {
    return frontmatter.title;
  }
  // Fallback: first # heading
  const match = content.match(/^#\s+(.+)$/m);
  return match ? match[1].trim() : "";
}

function getUrlPath(filePath: string, srcDir: string): string {
  let rel = path.relative(srcDir, filePath).replace(/\\/g, "/");
  rel = rel.replace(/\.md$/, "");
  if (rel === "index") return "/";
  if (rel.endsWith("/index")) rel = rel.slice(0, -"/index".length);
  return "/" + rel;
}

function computeOgImageUrl(
  urlPath: string,
  base: string,
  siteUrl?: string,
  generateOgImage?: boolean,
  staticOgImage?: string,
): string {
  if (!generateOgImage) return staticOgImage || "";

  const cleanBase = base.endsWith("/") ? base : base + "/";
  let relativePath: string;
  if (urlPath === "/") {
    relativePath = `${cleanBase}og-image.png`;
  } else {
    relativePath = `${cleanBase}${urlPath.replace(/^\//, "")}/og-image.png`;
  }

  if (siteUrl) {
    const cleanSiteUrl = siteUrl.replace(/\/$/, "");
    return `${cleanSiteUrl}${relativePath}`;
  }
  return relativePath;
}

function validatePage(
  page: { title: string; description: string; ogImageUrl: string },
  options: ResolvedOptions,
): { level: "error" | "warning"; message: string }[] {
  const warnings: { level: "error" | "warning"; message: string }[] = [];

  if (!page.title) {
    warnings.push({ level: "error", message: "title is missing" });
  } else if (page.title.length > 70) {
    warnings.push({ level: "warning", message: `title is too long (${page.title.length}/70)` });
  }

  if (!page.description) {
    warnings.push({ level: "warning", message: "description is missing" });
  } else if (page.description.length > 200) {
    warnings.push({
      level: "warning",
      message: `description is too long (${page.description.length}/200)`,
    });
  }

  const generateOgImage = options.ogImage || options.ssg.generateOgImage;
  if (generateOgImage && !options.ssg.siteUrl) {
    warnings.push({ level: "warning", message: "ogImage enabled but siteUrl is not set" });
  }

  return warnings;
}

async function collectPages(options: ResolvedOptions, root: string): Promise<PageOgData[]> {
  const srcDir = path.resolve(root, options.srcDir);
  const files = await glob("**/*.md", { cwd: srcDir, absolute: true });

  const pages: PageOgData[] = [];
  const generateOgImage = options.ogImage || options.ssg.generateOgImage;

  for (const file of files.sort()) {
    const content = fs.readFileSync(file, "utf-8");
    const frontmatter = parseFrontmatter(content);

    // Skip entry layout pages (they are landing pages, not content pages)
    if (frontmatter.layout === "entry") continue;

    const title = extractTitle(content, frontmatter);
    const description = typeof frontmatter.description === "string" ? frontmatter.description : "";
    const author = typeof frontmatter.author === "string" ? frontmatter.author : "";
    const tags = Array.isArray(frontmatter.tags)
      ? (frontmatter.tags as string[])
      : typeof frontmatter.tags === "string"
        ? [frontmatter.tags]
        : [];

    const urlPath = getUrlPath(file, srcDir);
    const ogImageUrl = computeOgImageUrl(
      urlPath,
      options.base,
      options.ssg.siteUrl,
      generateOgImage,
      options.ssg.ogImage,
    );

    const page = {
      path: path.relative(srcDir, file),
      urlPath,
      title,
      description,
      author,
      tags,
      ogImageUrl,
      warnings: [] as PageOgData["warnings"],
    };
    page.warnings = validatePage(page, options);
    pages.push(page);
  }

  return pages;
}

// =============================================================================
// HTML Rendering
// =============================================================================

function renderViewerHtml(pages: PageOgData[], options: ResolvedOptions): string {
  const generateOgImage = options.ogImage || options.ssg.generateOgImage;
  const totalWarnings = pages.reduce(
    (sum, p) => sum + p.warnings.filter((w) => w.level === "warning").length,
    0,
  );
  const totalErrors = pages.reduce(
    (sum, p) => sum + p.warnings.filter((w) => w.level === "error").length,
    0,
  );

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>OG Viewer - ox-content</title>
  <style>
    :root {
      --bg: #ffffff;
      --bg-card: #f5f7fb;
      --bg-preview: #ffffff;
      --text: #131a30;
      --text-muted: #4f607b;
      --border: #d2dbea;
      --accent: #4f6fae;
      --accent-light: #eef2fa;
      --error: #dc2626;
      --error-bg: #fef2f2;
      --warning: #d97706;
      --warning-bg: #fffbeb;
      --success: #16a34a;
      --tag-bg: #ecf3ff;
      --radius: 16px;
    }
    @media (prefers-color-scheme: dark) {
      :root {
        --bg: #060816;
        --bg-card: #0d1528;
        --bg-preview: #10172d;
        --text: #ebf2ff;
        --text-muted: #8ea0bf;
        --border: #223252;
        --accent: #86a4da;
        --accent-light: #151730;
        --error: #f87171;
        --error-bg: #450a0a;
        --warning: #fbbf24;
        --warning-bg: #451a03;
        --success: #4ade80;
        --tag-bg: #131b33;
      }
    }
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body { font-family: 'IBM Plex Sans', 'Avenir Next', 'Segoe UI', system-ui, sans-serif; background: radial-gradient(circle at top left, rgba(79,111,174,0.10), transparent 24%), radial-gradient(circle at 85% 14%, rgba(145,237,233,0.08), transparent 22%), var(--bg); color: var(--text); }
    .header { padding: 16px 24px; border-bottom: 1px solid var(--border); display: flex; align-items: center; gap: 12px; }
    .header svg { width: 28px; height: 28px; color: var(--accent); }
    .header h1 { font-size: 18px; font-weight: 600; }
    .header h1 span { color: var(--text-muted); font-weight: 400; }
    .header-actions { margin-left: auto; }
    .btn { padding: 6px 14px; border: 1px solid var(--border); border-radius: 6px; background: var(--bg-card); color: var(--text); cursor: pointer; font-size: 13px; transition: all 0.15s; }
    .btn:hover { border-color: var(--accent); color: var(--accent); }
    .summary { padding: 12px 24px; display: flex; gap: 20px; border-bottom: 1px solid var(--border); font-size: 13px; color: var(--text-muted); flex-wrap: wrap; align-items: center; }
    .summary-item { display: flex; align-items: center; gap: 4px; }
    .summary-item strong { color: var(--text); }
    .summary-dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; }
    .dot-error { background: var(--error); }
    .dot-warning { background: var(--warning); }
    .dot-success { background: var(--success); }
    .toolbar { padding: 12px 24px; display: flex; gap: 8px; border-bottom: 1px solid var(--border); flex-wrap: wrap; align-items: center; }
    .filter-btn { padding: 4px 12px; border: 1px solid var(--border); border-radius: 16px; background: transparent; color: var(--text-muted); cursor: pointer; font-size: 12px; transition: all 0.15s; }
    .filter-btn.active { background: var(--accent); color: #fff; border-color: var(--accent); }
    .search-input { padding: 6px 12px; border: 1px solid var(--border); border-radius: 6px; background: var(--bg); color: var(--text); font-size: 13px; flex: 1; min-width: 200px; }
    .search-input::placeholder { color: var(--text-muted); }
    .container { padding: 24px; display: flex; flex-direction: column; gap: 20px; max-width: 1200px; margin: 0 auto; }
    .card { border: 1px solid var(--border); border-radius: var(--radius); background: var(--bg-card); overflow: hidden; }
    .card-header { padding: 16px; border-bottom: 1px solid var(--border); }
    .card-path { font-size: 12px; color: var(--text-muted); font-family: monospace; margin-bottom: 4px; }
    .card-title { font-size: 16px; font-weight: 600; }
    .card-desc { font-size: 13px; color: var(--text-muted); margin-top: 4px; }
    .card-meta { display: flex; gap: 8px; margin-top: 8px; flex-wrap: wrap; align-items: center; }
    .tag { padding: 2px 8px; background: var(--tag-bg); border-radius: 4px; font-size: 11px; color: var(--text-muted); }
    .card-warnings { padding: 8px 16px; display: flex; flex-direction: column; gap: 4px; }
    .warning-item { font-size: 12px; padding: 4px 8px; border-radius: 4px; }
    .warning-item.error { background: var(--error-bg); color: var(--error); }
    .warning-item.warning { background: var(--warning-bg); color: var(--warning); }
    .card-previews { padding: 16px; display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
    @media (max-width: 768px) { .card-previews { grid-template-columns: 1fr; } }
    .preview { border: 1px solid var(--border); border-radius: 6px; overflow: hidden; }
    .preview-label { padding: 6px 10px; font-size: 11px; font-weight: 600; color: var(--text-muted); background: var(--bg); border-bottom: 1px solid var(--border); text-transform: uppercase; letter-spacing: 0.5px; }
    .preview-card { background: var(--bg-preview); }
    .preview-img { width: 100%; aspect-ratio: 1200/630; background: linear-gradient(135deg, var(--accent-light), var(--bg-card)); display: flex; align-items: center; justify-content: center; color: var(--text-muted); font-size: 12px; overflow: hidden; }
    .preview-img img { width: 100%; height: 100%; object-fit: cover; }
    .preview-body { padding: 10px 12px; }
    .preview-url { font-size: 11px; color: var(--text-muted); margin-bottom: 2px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    .preview-title { font-size: 14px; font-weight: 600; line-height: 1.3; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
    .preview-desc { font-size: 12px; color: var(--text-muted); margin-top: 2px; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
    .empty { text-align: center; padding: 60px; color: var(--text-muted); }
    .spin { animation: spin 0.6s linear infinite; }
    @keyframes spin { to { transform: rotate(360deg); } }
  </style>
</head>
<body>
  <div class="header">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
    <h1>OG Viewer <span>/ ox-content</span></h1>
    <div class="header-actions">
      <button class="btn" id="refresh-btn" onclick="refresh()">Refresh</button>
    </div>
  </div>
  <div class="summary" id="summary">
    <div class="summary-item"><strong id="s-pages">${pages.length}</strong>&nbsp;pages</div>
    <div class="summary-item"><span class="summary-dot dot-error"></span>&nbsp;<strong id="s-errors">${totalErrors}</strong>&nbsp;errors</div>
    <div class="summary-item"><span class="summary-dot dot-warning"></span>&nbsp;<strong id="s-warnings">${totalWarnings}</strong>&nbsp;warnings</div>
    <div class="summary-item"><span class="summary-dot ${generateOgImage ? "dot-success" : "dot-warning"}"></span>&nbsp;ogImage: <strong>${generateOgImage ? "enabled" : "disabled"}</strong></div>
  </div>
  <div class="toolbar">
    <button class="filter-btn active" data-filter="all" onclick="setFilter('all')">All</button>
    <button class="filter-btn" data-filter="warnings" onclick="setFilter('warnings')">Warnings</button>
    <button class="filter-btn" data-filter="errors" onclick="setFilter('errors')">Errors</button>
    <input class="search-input" type="text" placeholder="Search pages..." oninput="applyFilters()" id="search-input">
  </div>
  <div class="container" id="container"></div>

  <script>
    let pages = ${JSON.stringify(pages)};
    let currentFilter = 'all';

    function setFilter(f) {
      currentFilter = f;
      document.querySelectorAll('.filter-btn').forEach(b => b.classList.toggle('active', b.dataset.filter === f));
      applyFilters();
    }

    function applyFilters() {
      const q = document.getElementById('search-input').value.toLowerCase();
      const filtered = pages.filter(p => {
        if (currentFilter === 'errors' && !p.warnings.some(w => w.level === 'error')) return false;
        if (currentFilter === 'warnings' && !p.warnings.length) return false;
        if (q && !p.path.toLowerCase().includes(q) && !p.title.toLowerCase().includes(q) && !p.description.toLowerCase().includes(q)) return false;
        return true;
      });
      renderCards(filtered);
    }

    function esc(s) {
      const d = document.createElement('div');
      d.textContent = s;
      return d.innerHTML;
    }

    function renderCards(list) {
      const c = document.getElementById('container');
      if (!list.length) {
        c.innerHTML = '<div class="empty">No pages match the current filter.</div>';
        return;
      }
      c.innerHTML = list.map(p => {
        const warnings = p.warnings.map(w =>
          '<div class="warning-item ' + w.level + '">' + (w.level === 'error' ? '\\u2716' : '\\u26A0') + ' ' + esc(w.message) + '</div>'
        ).join('');
        const tags = p.tags.map(t => '<span class="tag">' + esc(t) + '</span>').join('');
        const author = p.author ? '<span class="tag">by ' + esc(p.author) + '</span>' : '';
        const imgHtml = p.ogImageUrl
          ? '<img src="' + esc(p.ogImageUrl) + '" onerror="this.parentNode.innerHTML=\\'No OG image\\'">'
          : 'No OG image';
        const siteHost = ${JSON.stringify(options.ssg.siteUrl || "example.com")};
        return '<div class="card">'
          + '<div class="card-header">'
          + '<div class="card-path">' + esc(p.path) + ' &rarr; ' + esc(p.urlPath) + '</div>'
          + '<div class="card-title">' + (esc(p.title) || '<em style="color:var(--error)">No title</em>') + '</div>'
          + (p.description ? '<div class="card-desc">' + esc(p.description) + '</div>' : '')
          + (tags || author ? '<div class="card-meta">' + author + tags + '</div>' : '')
          + '</div>'
          + (warnings ? '<div class="card-warnings">' + warnings + '</div>' : '')
          + '<div class="card-previews">'
          + '<div class="preview"><div class="preview-label">Twitter (summary_large_image)</div><div class="preview-card"><div class="preview-img">' + imgHtml + '</div><div class="preview-body"><div class="preview-url">' + esc(siteHost) + '</div><div class="preview-title">' + esc(p.title) + '</div><div class="preview-desc">' + esc(p.description) + '</div></div></div></div>'
          + '<div class="preview"><div class="preview-label">Facebook (Open Graph)</div><div class="preview-card"><div class="preview-img">' + imgHtml + '</div><div class="preview-body"><div class="preview-url">' + esc(siteHost) + '</div><div class="preview-title">' + esc(p.title) + '</div><div class="preview-desc">' + esc(p.description) + '</div></div></div></div>'
          + '</div>'
          + '</div>';
      }).join('');
    }

    async function refresh() {
      const btn = document.getElementById('refresh-btn');
      btn.textContent = 'Refreshing...';
      btn.disabled = true;
      try {
        const res = await fetch('/__og-viewer/api/pages');
        pages = await res.json();
        updateSummary();
        applyFilters();
      } catch(e) {
        console.error('Refresh failed:', e);
      } finally {
        btn.textContent = 'Refresh';
        btn.disabled = false;
      }
    }

    function updateSummary() {
      document.getElementById('s-pages').textContent = pages.length;
      document.getElementById('s-errors').textContent = pages.reduce((s,p) => s + p.warnings.filter(w => w.level === 'error').length, 0);
      document.getElementById('s-warnings').textContent = pages.reduce((s,p) => s + p.warnings.filter(w => w.level === 'warning').length, 0);
    }

    renderCards(pages);
  </script>
</body>
</html>`;
}

// =============================================================================
// Plugin
// =============================================================================

export function createOgViewerPlugin(options: ResolvedOptions): Plugin {
  return {
    name: "ox-content:og-viewer",
    apply: "serve",

    configureServer(server) {
      server.middlewares.use(async (req, res, next) => {
        if (req.url === "/__og-viewer" || req.url === "/__og-viewer/") {
          const root = server.config.root || process.cwd();
          try {
            const pages = await collectPages(options, root);
            const html = renderViewerHtml(pages, options);
            res.setHeader("Content-Type", "text/html; charset=utf-8");
            res.end(html);
          } catch (err) {
            res.statusCode = 500;
            res.end(`OG Viewer error: ${err instanceof Error ? err.message : String(err)}`);
          }
          return;
        }

        if (req.url === "/__og-viewer/api/pages") {
          const root = server.config.root || process.cwd();
          try {
            const pages = await collectPages(options, root);
            res.setHeader("Content-Type", "application/json; charset=utf-8");
            res.end(JSON.stringify(pages));
          } catch (err) {
            res.statusCode = 500;
            res.end(JSON.stringify({ error: String(err) }));
          }
          return;
        }

        next();
      });
    },
  };
}
