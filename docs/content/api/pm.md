# pm.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/pm.ts)**

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
  <strong>2</strong>
  <span>parameters</span>
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

<details id="pm" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">pm</code><span class="ox-api-entry__description">Package Manager Tabs Plugin Transforms &lt;pm&gt;npm install …&lt;/pm&gt; blocks into a tab group with one tab per package manager…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Package Manager Tabs Plugin</p>
<p>Transforms &lt;pm&gt;npm install …&lt;/pm&gt; blocks into a tab group with one tab per package manager (npm/pnpm/yarn/bun). The single npm-style command is converted to each package manager&#39;s equivalent natively in Rust (<code>transformPmEmbeds</code> in</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/pm.ts#L1-L16" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--tags">
<h4>Tags</h4>
<ul class="ox-api-entry__tags"><li><span class="ox-api-entry__tag-name">@ox-content</span><span class="ox-api-entry__tag-value">/napi), and the result reuses the same <code>ox-tabs</code> widget markup as<br>the generic <code>&lt;tabs&gt;</code> plugin so styling and keyboard navigation are consistent.<br><br>Syncing is opt-in (off by default): when enabled, the rendered group carries a<br><code>data-ox-tab-group=&quot;pkg-manager&quot;</code> attribute so the client runtime can keep<br>every package-manager group on the page in sync via localStorage.<br><br>Package-manager groups share the tab-group counter with the <code>&lt;tabs&gt;</code> plugin so<br><code>data-group</code> ids (and the CSS produced by <code>generateTabsCSS</code>) stay unique.</span></li></ul>
</div>
  </div>
</details>

<details id="pmoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">PmOptions</code><span class="ox-api-entry__description">Options for transformPm.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for <a href="#transformpm">transformPm</a>.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface PmOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/pm.ts#L22-L31" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="pmoptions-sync">
  <td><code>sync</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable opt-in synced package-manager tab groups. When <code>true</code>, a<br><code>data-ox-tab-group=&quot;pkg-manager&quot;</code> attribute is emitted so the client runtime<br>syncs the active package manager across every pm group on the page and<br>persists the choice in localStorage.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="transformpm" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">transformPm(html: string, options?: PmOptions): Promise&lt;string&gt;</code><span class="ox-api-entry__description">Transform &lt;pm&gt; package-manager blocks in HTML into install tabs.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;string&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Transform <code>&lt;pm&gt;</code> package-manager blocks in HTML into install tabs.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function transformPm(html: string, options?: PmOptions): Promise&lt;string&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/pm.ts#L40-L55" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">html</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  <p class="ox-api-entry__param-description">Rendered HTML potentially containing <code>&lt;pm&gt;</code> blocks.</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="#pmoptions">PmOptions</a></code>
  </div>
  <p class="ox-api-entry__param-description">Package-manager tab options (syncing is opt-in). — optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;string&gt;</code>
  <p class="ox-api-entry__return-description">The rewritten HTML.</p>
</div>
</div>
  </div>
</details>

