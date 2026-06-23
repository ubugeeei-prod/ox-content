# browser.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/browser.ts)**

> 3 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>3</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>returns</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="browser" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">browser</code><span class="ox-api-entry__description">Chromium browser session with automatic cleanup via Explicit Resource Management. Usage: await using session = await op…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Chromium browser session with automatic cleanup via Explicit Resource Management.</p>
<p>Usage: await using session = await openBrowser(); const png = await session.renderPage(html, 1200, 630); // browser.close() is called automatically when session goes out of scope</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/browser.ts#L1-L8" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="ogbrowsersession" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">OgBrowserSession extends AsyncDisposable</code><span class="ox-api-entry__description">A browser session that can render HTML pages to PNG. Implements AsyncDisposable…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>A browser session that can render HTML pages to PNG. Implements AsyncDisposable for automatic cleanup via <code>await using</code>.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface OgBrowserSession extends AsyncDisposable</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/browser.ts#L22-L24" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group ox-api-entry__member-group--details">
<h5>Methods</h5>
<div class="ox-api-entry__member-details">
<section id="ogbrowsersession-renderpage" class="ox-api-entry__member-detail">
<h5>renderPage()</h5>
<pre><code class="language-typescript">renderPage(html: string, width: number, height: number, publicDir?: string): Promise&lt;Buffer&gt;;</code></pre>
<div class="ox-api-entry__member-detail-section ox-api-entry__member-detail-section--params">
<h6>Parameters</h6>
<ul class="ox-api-entry__member-params"><li><code>html</code></li><li><code>width</code></li><li><code>height</code></li><li><code>publicDir</code> optional</li></ul>
</div><div class="ox-api-entry__member-detail-section ox-api-entry__member-detail-section--returns">
<h6>Returns</h6>
<code class="ox-api-entry__return-type">Promise&lt;Buffer&gt;</code>
</div></section>
</div>
</div>
</div>
  </div>
</details>

<details id="openbrowser" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">openBrowser(): Promise&lt;OgBrowserSession | null&gt;</code><span class="ox-api-entry__description">Opens a Chromium browser and returns a session for rendering OG images. Returns…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">returns Promise&lt;OgBrowserSession | null&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opens a Chromium browser and returns a session for rendering OG images. Returns null if Playwright/Chromium is not available.</p>
<p>The session implements AsyncDisposable — use <code>await using</code> for automatic cleanup:</p>
<pre><code class="language-ts">await using session = await openBrowser();
if (!session) return;
const png = await session.renderPage(html, 1200, 630);</code></pre>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function openBrowser(): Promise&lt;OgBrowserSession | null&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/browser.ts#L37-L77" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;<a href="#ogbrowsersession">OgBrowserSession</a> | null&gt;</code>
  
</div>
</div>
  </div>
</details>
