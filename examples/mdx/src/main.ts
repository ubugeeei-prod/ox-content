/**
 * Built-in MDX example — import `.mdx` and a sibling `.md` file.
 */

import mdx from "./content/index.mdx";
import plain from "./content/plain.md";

function tocList(entries: OxContentTocEntry[]): string {
  return entries
    .map(
      (entry) => `
        <li>
          <a href="#${entry.slug}">${entry.text}</a>
          ${entry.children.length > 0 ? `<ul>${tocList(entry.children)}</ul>` : ""}
        </li>
      `,
    )
    .join("");
}

function escapeHtml(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}

const app = document.getElementById("app");
if (app) {
  app.innerHTML = `
    <nav class="toc">
      <h2>On this page</h2>
      <ul>${tocList(mdx.toc)}</ul>
      <p class="toc-note">Also see the sibling <a href="#plain-markdown">.md contrast</a>.</p>
    </nav>
    <div class="pages">
      <main class="content">
        ${mdx.html}
        <section class="html-dump">
          <h2>Generated HTML from <code>index.mdx</code></h2>
          <p>
            Look for <code>data-ox-island="NoteCard"</code>, the lowercase
            <code>&lt;note&gt;</code> markup, and the absence of
            <code>import</code> / <code>export</code> and <code>{name}</code>.
          </p>
          <pre><code>${escapeHtml(mdx.html)}</code></pre>
        </section>
        <section class="content" id="plain-markdown">
          ${plain.html}
        </section>
      </main>
    </div>
  `;
}

if (import.meta.hot) {
  import.meta.hot.on("ox-content:update", (data) => {
    console.log("Content updated:", data.file);
    import.meta.hot?.invalidate();
  });
}
