# browser.ts

**[Source](https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/browser.ts)**

> 2 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="ogbrowsersession" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">OgBrowserSession extends AsyncDisposable</code><span class="ox-api-entry__description">A browser session that can render HTML pages to PNG. Implements AsyncDisposable…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>A browser session that can render HTML pages to PNG. Implements AsyncDisposable for automatic cleanup via <code>await using</code>.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/browser.ts#L17-L19">View source</a></p>
  </div>
</details>

<details id="openbrowser" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">openBrowser(): Promise&lt;OgBrowserSession | null&gt;</code><span class="ox-api-entry__description">Opens a Chromium browser and returns a session for rendering OG images. Returns…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opens a Chromium browser and returns a session for rendering OG images. Returns null if Playwright/Chromium is not available. The session implements AsyncDisposable — use <code>await using</code> for automatic cleanup:</p>
<pre><code class="language-ts">await using session = await openBrowser();
if (!session) return;
const png = await session.renderPage(html, 1200, 630);</code></pre>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/browser.ts#L32-L75">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise</code>
  
</div>
</div>
  </div>
</details>
