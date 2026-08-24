# jsx-dev-runtime.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-dev-runtime.ts)**

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
  <strong>6</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>returns</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>examples</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="jsx-dev-runtime" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">jsx-dev-runtime</code><span class="ox-api-entry__description">Development-runtime entry point for the static HTML JSX transform. TypeScript&#39;s react-jsxdev transform (and Vite&#39;s dev-…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Development-runtime entry point for the static HTML JSX transform.</p>
<p>TypeScript&#39;s <code>react-jsxdev</code> transform (and Vite&#39;s dev-mode JSX transform) import from <code>&lt;jsxImportSource&gt;/jsx-dev-runtime</code> and call <code>jsxDEV</code> with the extra debug arguments below. This runtime renders static HTML, so there is nothing useful to do with them — they are accepted and ignored so the same templates compile in both dev and build.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-dev-runtime.ts#L1-L14" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-json">{ &quot;compilerOptions&quot;: { &quot;jsx&quot;: &quot;react-jsxdev&quot;, &quot;jsxImportSource&quot;: &quot;@ox-content/vite-plugin&quot; } }</code></pre>
</div>
</div>
  </div>
</details>

<details id="jsxdev" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">jsxDEV(type: JSXElementType, props: JSXProps, key?: string, _isStaticChildren?: boolean, _source?: JSXSource, _self?: unknown): JSXNode</code><span class="ox-api-entry__description">Creates a JSX element in development mode. The transform passes isStaticChildre…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 params</span><span class="ox-api-badge">returns JSXNode</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Creates a JSX element in development mode.</p>
<p>The transform passes <code>isStaticChildren</code>, <code>source</code> and <code>self</code> for React&#39;s dev warnings; static HTML generation has no use for them.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function jsxDEV(type: JSXElementType, props: JSXProps, key?: string, _isStaticChildren?: boolean, _source?: JSXSource, _self?: unknown): JSXNode</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-dev-runtime.ts#L36-L45" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">type</code>
    <code class="ox-api-entry__param-type"><a href="./jsx-html.md#jsxelementtype">JSXElementType</a></code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">props</code>
    <code class="ox-api-entry__param-type"><a href="./jsx-html.md#jsxprops">JSXProps</a></code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">key</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">_isStaticChildren</code>
    <code class="ox-api-entry__param-type">boolean</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">_source</code>
    <code class="ox-api-entry__param-type"><a href="#jsxsource">JSXSource</a></code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">_self</code>
    <code class="ox-api-entry__param-type">unknown</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="./jsx-html.md#jsxnode">JSXNode</a></code>
  
</div>
</div>
  </div>
</details>

<details id="jsxsource" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">JSXSource</code><span class="ox-api-entry__description">Source position the dev transform attaches to each element.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Source position the dev transform attaches to each element.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface JSXSource</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-dev-runtime.ts#L24-L28" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="jsxsource-columnnumber">
  <td><code>columnNumber</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td></td>
</tr>
<tr id="jsxsource-filename">
  <td><code>fileName</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="jsxsource-linenumber">
  <td><code>lineNumber</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

