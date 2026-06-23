# mermaid.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid.ts)**

> 4 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>4</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>variables</span>
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
  <span>returns</span>
</span>
<span class="ox-api-stat ox-api-stat--warning">
  <strong>1</strong>
  <span>deprecated</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="mermaid" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">mermaid</code><span class="ox-api-entry__description">Mermaid Plugin - Native Rust renderer via NAPI Renders mermaid code blocks to SVG using the native Rust renderer via NA…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Mermaid Plugin - Native Rust renderer via NAPI</p>
<p>Renders mermaid code blocks to SVG using the native Rust renderer via NAPI. Delegates to the NAPI <code>transformMermaid</code> function which extracts mermaid code blocks from HTML and renders them using mmdc.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid.ts#L1-L7" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="mermaidclientscript" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">variable</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">const mermaidClientScript = &quot;&quot;</code><span class="ox-api-entry__meta"><span class="ox-api-badge ox-api-badge--warning">deprecated</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export const mermaidClientScript = &quot;&quot;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid.ts#L136" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="napibindings" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">variable</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">let napiBindings: { transformMermaid: (html: string, mmdcPath: string) =&gt; { html: string; errors: string[] } } | null</code><span class="ox-api-entry__description">Cached NAPI bindings</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Cached NAPI bindings</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">let napiBindings: { transformMermaid: (html: string, mmdcPath: string) =&gt; { html: string; errors: string[] } } | null</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid.ts#L23-L25" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="transformmermaidstatic" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">transformMermaidStatic(html: string, _options?: MermaidOptions): Promise&lt;string&gt;</code><span class="ox-api-entry__description">Transforms mermaid code blocks in HTML to rendered SVG diagrams. Uses the nativ…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;string&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Transforms mermaid code blocks in HTML to rendered SVG diagrams. Uses the native Rust NAPI transformMermaid function.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function transformMermaidStatic(html: string, _options?: MermaidOptions): Promise&lt;string&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid.ts#L97-L122" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
    <code class="ox-api-entry__param-name">_options</code>
    <code class="ox-api-entry__param-type">MermaidOptions</code>
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
