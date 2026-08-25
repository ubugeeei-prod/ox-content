# mermaid-protect.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid-protect.ts)**

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
  <strong>3</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
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

<details id="mermaid-protect" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">mermaid-protect</code><span class="ox-api-entry__description">Protects mermaid SVG content from rehype HTML5 parser corruption. rehypeParse + rehypeStringify converts &lt;br /&gt; in SVG…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Protects mermaid SVG content from rehype HTML5 parser corruption.</p>
<p>rehypeParse + rehypeStringify converts <code>&lt;br /&gt;</code> in SVG foreignObject to <code>&lt;br&gt;&lt;/br&gt;</code>, which HTML5 interprets as 2 &lt;br&gt; elements. Each rehype pass doubles them: 1 → 2 → 4 → 8 → 16.</p>
<p>This module extracts ox-mermaid SVG blocks into placeholders before rehype processing and restores them after.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid-protect.ts#L1-L10" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="mermaidsvgprotection" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">MermaidSvgProtection</code><span class="ox-api-entry__description">Protects mermaid SVG content from rehype HTML5 parser corruption. rehypeParse +…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Protects mermaid SVG content from rehype HTML5 parser corruption.</p>
<p>rehypeParse + rehypeStringify converts <code>&lt;br /&gt;</code> in SVG foreignObject to <code>&lt;br&gt;&lt;/br&gt;</code>, which HTML5 interprets as 2 &lt;br&gt; elements. Each rehype pass doubles them: 1 → 2 → 4 → 8 → 16.</p>
<p>This module extracts ox-mermaid SVG blocks into placeholders before rehype processing and restores them after.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface MermaidSvgProtection</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid-protect.ts#L12-L15" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="mermaidsvgprotection-html">
  <td><code>html</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="mermaidsvgprotection-svgs">
  <td><code>svgs</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Map&lt;string, string&gt;</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="protectmermaidsvgs" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">protectMermaidSvgs(html: string): MermaidSvgProtection</code><span class="ox-api-entry__description">Extract &lt;div class=&quot;ox-mermaid&quot;&gt;...&lt;/div&gt; blocks and replace with HTML comment…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns MermaidSvgProtection</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Extract <code>&lt;div class=&quot;ox-mermaid&quot;&gt;...&lt;/div&gt;</code> blocks and replace with HTML comment placeholders that rehype will preserve.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function protectMermaidSvgs(html: string): MermaidSvgProtection</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid-protect.ts#L21-L64" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">html</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="#mermaidsvgprotection">MermaidSvgProtection</a></code>
  
</div>
</div>
  </div>
</details>

<details id="restoremermaidsvgs" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">restoreMermaidSvgs(html: string, svgs: Map&lt;string, string&gt;): string</code><span class="ox-api-entry__description">Restore protected mermaid SVG blocks from placeholders.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Restore protected mermaid SVG blocks from placeholders.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function restoreMermaidSvgs(html: string, svgs: Map&lt;string, string&gt;): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid-protect.ts#L69-L81" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
    <code class="ox-api-entry__param-name">svgs</code>
    <code class="ox-api-entry__param-type">Map&lt;string, string&gt;</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string</code>
  
</div>
</div>
  </div>
</details>

