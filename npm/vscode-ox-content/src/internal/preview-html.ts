/**
 * HTML helpers used by the preview panel. Pure functions — no `vscode`
 * imports — so they can be unit-tested under plain Node.
 */

/**
 * Escape `&`, `<`, and `>` in a string so it is safe to interpolate
 * inside an HTML text node or attribute value. Order matters: `&` must
 * be escaped first so the subsequent escapes don't double-encode it.
 */
export function escapeHtml(text: string): string {
  return text.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}

/**
 * Render the fallback HTML shown in the preview webview when the LSP
 * request fails or returns an empty payload. The output is fully
 * self-contained (inline styles, no scripts) so it survives the
 * `enableScripts: false` panel option.
 */
export function errorHtml(message: string): string {
  const escaped = escapeHtml(message);

  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style>
      body { margin: 0; padding: 24px; font: 14px/1.6 ui-sans-serif, system-ui, sans-serif; background: #0f172a; color: #e2e8f0; }
      .card { max-width: 720px; margin: 0 auto; padding: 20px 22px; border-radius: 16px; border: 1px solid #334155; background: #111827; }
      h1 { margin-top: 0; font-size: 1rem; }
      code { color: #fda4af; }
    </style>
  </head>
  <body>
    <div class="card">
      <h1>Ox Content Preview</h1>
      <p>${escaped}</p>
      <p>Make sure <code>ox-content-lsp</code> is available or set <code>oxContent.server.path</code>.</p>
    </div>
  </body>
</html>`;
}
