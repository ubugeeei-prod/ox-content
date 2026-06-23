# index.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/index.ts)**

> 4 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>4</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
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
  <strong>12</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>13</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>returns</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="index" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">index</code><span class="ox-api-entry__description">ox-content Built-in Plugins All plugins are designed with No-JavaScript-First principle. They generate static HTML at b…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>ox-content Built-in Plugins</p>
<p>All plugins are designed with No-JavaScript-First principle. They generate static HTML at build time and require no client-side JS.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/index.ts#L1-L6" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="transformalloptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">TransformAllOptions</code><span class="ox-api-entry__description">Transform all plugin components in HTML. Call this during SSG build to process…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">13 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Transform all plugin components in HTML. Call this during SSG build to process all plugins at once.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface TransformAllOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/index.ts#L58-L77" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="transformalloptions-bluesky">
  <td><code>bluesky</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="transformalloptions-github">
  <td><code>github</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | GitHubOptions</code></td>
  <td></td>
</tr>
<tr id="transformalloptions-githubtoken">
  <td><code>githubToken</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="transformalloptions-mermaid">
  <td><code>mermaid</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="transformalloptions-ogp">
  <td><code>ogp</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | OgpOptions</code></td>
  <td></td>
</tr>
<tr id="transformalloptions-opengraph">
  <td><code>openGraph</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | OgpOptions</code></td>
  <td></td>
</tr>
<tr id="transformalloptions-pm">
  <td><code>pm</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="./pm.md#pmoptions">PmOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Expand <code>&lt;pm&gt;</code> package-manager blocks into install tabs. Pass an object to<br>opt in to synced groups (<code>{ sync: true }</code>); syncing is off by default.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="transformalloptions-spotify">
  <td><code>spotify</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="transformalloptions-stackblitz">
  <td><code>stackBlitz</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="transformalloptions-tabs">
  <td><code>tabs</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="transformalloptions-twitter">
  <td><code>twitter</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="transformalloptions-webcontainer">
  <td><code>webContainer</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="transformalloptions-youtube">
  <td><code>youtube</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="transformallplugins" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">transformAllPlugins(html: string, options: TransformAllOptions = {}): Promise&lt;string&gt;</code><span class="ox-api-entry__description">Transform all enabled plugins in HTML content.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;string&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Transform all enabled plugins in HTML content.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function transformAllPlugins(html: string, options: TransformAllOptions = {}): Promise&lt;string&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/index.ts#L82-L157" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">html</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="#transformalloptions">TransformAllOptions</a></code>
  </div>
  <p class="ox-api-entry__param-description">optional · default: {}</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;string&gt;</code>
  
</div>
</div>
  </div>
</details>

<details id="transformbuiltinembeds" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">transformBuiltinEmbeds(html: string, options: { github: GitHubOptions | false; openGraph: OgpOptions | false; pm?: PmOptions | false; spotify?: boolean; stackBlitz?: boolean; twitter?: boolean; bluesky?: boolean; webContainer?: boolean; }): Promise&lt;string&gt;</code><span class="ox-api-entry__description">Transform built-in embed components in HTML content.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">10 params</span><span class="ox-api-badge">returns Promise&lt;string&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Transform built-in embed components in HTML content.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function transformBuiltinEmbeds(html: string, options: {
    github: GitHubOptions | false;
    openGraph: OgpOptions | false;
    pm?: PmOptions | false;
    spotify?: boolean;
    stackBlitz?: boolean;
    twitter?: boolean;
    bluesky?: boolean;
    webContainer?: boolean;
  }): Promise&lt;string&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/index.ts#L162-L208" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">html</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">{ github: GitHubOptions | false; openGraph: OgpOptions | false; <a href="./pm.md#pm">pm</a>?: <a href="./pm.md#pmoptions">PmOptions</a> | false; spotify?: boolean; stackBlitz?: boolean; twitter?: boolean; bluesky?: boolean; webContainer?: boolean }</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options.github</code>
    <code class="ox-api-entry__param-type">GitHubOptions | false</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options.openGraph</code>
    <code class="ox-api-entry__param-type">OgpOptions | false</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options.pm?</code>
    <code class="ox-api-entry__param-type"><a href="./pm.md#pmoptions">PmOptions</a> | false</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options.spotify?</code>
    <code class="ox-api-entry__param-type">boolean</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options.stackBlitz?</code>
    <code class="ox-api-entry__param-type">boolean</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options.twitter?</code>
    <code class="ox-api-entry__param-type">boolean</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options.bluesky?</code>
    <code class="ox-api-entry__param-type">boolean</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options.webContainer?</code>
    <code class="ox-api-entry__param-type">boolean</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;string&gt;</code>
  
</div>
</div>
  </div>
</details>
