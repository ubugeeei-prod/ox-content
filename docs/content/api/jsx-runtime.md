# jsx-runtime.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts)**

> 17 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>17</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>11</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>types</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>variables</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>19</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>11</strong>
  <span>returns</span>
</span>
<span class="ox-api-stat">
  <strong>5</strong>
  <span>examples</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="each" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">each&lt;T&gt;(items: T[], render: (item: T, index: number) =&gt; JSXNode): JSXNode</code><span class="ox-api-entry__description">Maps over an array and renders each item.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns JSXNode</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Maps over an array and renders each item.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function each&lt;T&gt;(items: T[], render: (item: T, index: number) =&gt; JSXNode): JSXNode</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L286-L289" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">items</code>
    <code class="ox-api-entry__param-type">T[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">render</code>
    <code class="ox-api-entry__param-type">(item: T, <a href="./index-module.md#index">index</a>: number) =&gt; <a href="#jsxnode">JSXNode</a></code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="#jsxnode">JSXNode</a></code>
  
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-tsx">{each(items, (item) =&gt; &lt;li&gt;{item.name}&lt;/li&gt;)}</code></pre>
</div>
</div>
  </div>
</details>

<details id="escapehtml" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">escapeHtml(str: string): string</code><span class="ox-api-entry__description">Escapes HTML special characters to prevent XSS.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Escapes HTML special characters to prevent XSS.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">function escapeHtml(str: string): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L78-L85" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">str</code>
    <code class="ox-api-entry__param-type">string</code>
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

<details id="fragment" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">Fragment({ children }: { children?: JSXChild }): JSXNode</code><span class="ox-api-entry__description">Fragment component - renders children without a wrapper element.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns JSXNode</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Fragment component - renders children without a wrapper element.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function Fragment({ children }: { children?: JSXChild }): JSXNode</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L242-L244" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">param</code>
    <code class="ox-api-entry__param-type">{ children?: <a href="#jsxchild">JSXChild</a> }</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">param.children?</code>
    <code class="ox-api-entry__param-type"><a href="#jsxchild">JSXChild</a></code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="#jsxnode">JSXNode</a></code>
  
</div>
</div>
  </div>
</details>

<details id="jsx" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">jsx(type: JSXElementType, props: JSXProps, _key?: string): JSXNode</code><span class="ox-api-entry__description">Creates a JSX element. This is the core function called by the JSX transform.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span><span class="ox-api-badge">returns JSXNode</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Creates a JSX element. This is the core function called by the JSX transform.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function jsx(type: JSXElementType, props: JSXProps, _key?: string): JSXNode</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L194-L229" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">type</code>
    <code class="ox-api-entry__param-type"><a href="#jsxelementtype">JSXElementType</a></code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">props</code>
    <code class="ox-api-entry__param-type"><a href="#jsxprops">JSXProps</a></code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">_key</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="#jsxnode">JSXNode</a></code>
  
</div>
</div>
  </div>
</details>

<details id="jsx-runtime" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">jsx-runtime</code><span class="ox-api-entry__description">Custom JSX Runtime for Static HTML Generation This module provides a JSX runtime that outputs static HTML strings. No R…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Custom JSX Runtime for Static HTML Generation</p>
<p>This module provides a JSX runtime that outputs static HTML strings. No React, no hydration, no client-side JavaScript - just pure HTML.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L1-L26" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-tsx">// tsconfig.json or vite.config.ts
{
  &quot;compilerOptions&quot;: {
    &quot;jsx&quot;: &quot;react-jsx&quot;,
    &quot;jsxImportSource&quot;: &quot;@ox-content/vite-plugin&quot;
  }
}

// MyComponent.tsx
export function Hero({ title }: { title: string }) {
  return (
    &lt;section class=&quot;hero&quot;&gt;
      &lt;h1&gt;{title}&lt;/h1&gt;
    &lt;/section&gt;
  );
}</code></pre>
</div>
</div>
  </div>
</details>

<details id="jsxchild" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">JSXChild = string | number | boolean | null | undefined | JSXNode | JSXChild[]</code><span class="ox-api-entry__description">Valid JSX child types.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Valid JSX child types.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type JSXChild = string | number | boolean | null | undefined | JSXNode | JSXChild[]</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L142" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="jsxelementtype" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">JSXElementType = string | ((props: Record&lt;string, unknown&gt;) =&gt; JSXNode)</code><span class="ox-api-entry__description">JSX element type - either a string (intrinsic) or a function component.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>JSX element type - either a string (intrinsic) or a function component.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type JSXElementType = string | ((props: Record&lt;string, unknown&gt;) =&gt; JSXNode)</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L137" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="jsxnode" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">JSXNode</code><span class="ox-api-entry__description">JSX node - the result of JSX expressions.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>JSX node - the result of JSX expressions.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface JSXNode</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L147-L149" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="jsxnode-__html">
  <td><code>__html</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="jsxprops" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">JSXProps</code><span class="ox-api-entry__description">Props with children.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Props with children.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface JSXProps</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L154-L157" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group ox-api-entry__member-group--indexable">
<h5>Indexable</h5>
<div class="ox-api-entry__member-details">
<section class="ox-api-entry__member-detail ox-api-entry__member-detail--indexable">
<pre><code class="language-ts">[key: string]: unknown</code></pre></section>
</div>
</div>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="jsxprops-children">
  <td><code>children</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#jsxchild">JSXChild</a></code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="jsxs" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">jsxs(type: JSXElementType, props: JSXProps, key?: string): JSXNode</code><span class="ox-api-entry__description">Creates a JSX element with static children. Called by the JSX transform for ele…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span><span class="ox-api-badge">returns JSXNode</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Creates a JSX element with static children. Called by the JSX transform for elements with multiple children.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function jsxs(type: JSXElementType, props: JSXProps, key?: string): JSXNode</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L235-L237" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">type</code>
    <code class="ox-api-entry__param-type"><a href="#jsxelementtype">JSXElementType</a></code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">props</code>
    <code class="ox-api-entry__param-type"><a href="#jsxprops">JSXProps</a></code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">key</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="#jsxnode">JSXNode</a></code>
  
</div>
</div>
  </div>
</details>

<details id="raw" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">raw(html: string): JSXNode</code><span class="ox-api-entry__description">Creates raw HTML without escaping. Use with caution - only for trusted content.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns JSXNode</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Creates raw HTML without escaping. Use with caution - only for trusted content.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function raw(html: string): JSXNode</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L262-L264" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
  <code class="ox-api-entry__return-type"><a href="#jsxnode">JSXNode</a></code>
  
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-tsx">&lt;div&gt;{raw(&#39;&lt;strong&gt;Bold&lt;/strong&gt;&#39;)}&lt;/div&gt;</code></pre>
</div>
</div>
  </div>
</details>

<details id="renderattr" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">renderAttr(name: string, value: unknown): string</code><span class="ox-api-entry__description">Renders an attribute value to a string.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Renders an attribute value to a string.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">function renderAttr(name: string, value: unknown): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L106-L132" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">name</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">value</code>
    <code class="ox-api-entry__param-type">unknown</code>
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

<details id="renderchildren" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">renderChildren(children: JSXChild): string</code><span class="ox-api-entry__description">Renders children to HTML string.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Renders children to HTML string.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">function renderChildren(children: JSXChild): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L162-L188" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">children</code>
    <code class="ox-api-entry__param-type"><a href="#jsxchild">JSXChild</a></code>
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

<details id="rendertostring" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">renderToString(node: JSXNode): string</code><span class="ox-api-entry__description">Renders a JSX node to an HTML string.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Renders a JSX node to an HTML string.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function renderToString(node: JSXNode): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L249-L251" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">node</code>
    <code class="ox-api-entry__param-type"><a href="#jsxnode">JSXNode</a></code>
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

<details id="tohtmlattr" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">toHtmlAttr(name: string): string</code><span class="ox-api-entry__description">Converts a camelCase attribute name to kebab-case for HTML. Special handling fo…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Converts a camelCase attribute name to kebab-case for HTML. Special handling for data-<em> and aria-</em> attributes.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">function toHtmlAttr(name: string): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L91-L101" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">name</code>
    <code class="ox-api-entry__param-type">string</code>
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

<details id="void_elements" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">variable</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">const VOID_ELEMENTS = new Set([ &quot;area&quot;, &quot;base&quot;, &quot;br&quot;, &quot;col&quot;, &quot;embed&quot;, &quot;hr&quot;, &quot;img&quot;, &quot;input&quot;, &quot;link&quot;, &quot;meta&quot;, &quot;param&quot;, &quot;source&quot;, &quot;track&quot;, &quot;wbr&quot;, ])</code><span class="ox-api-entry__description">Custom JSX Runtime for Static HTML Generation This module provides a JSX runtim…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Custom JSX Runtime for Static HTML Generation</p>
<p>This module provides a JSX runtime that outputs static HTML strings. No React, no hydration, no client-side JavaScript - just pure HTML.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">const VOID_ELEMENTS = new Set([
  &quot;area&quot;,
  &quot;base&quot;,
  &quot;br&quot;,
  &quot;col&quot;,
  &quot;embed&quot;,
  &quot;hr&quot;,
  &quot;img&quot;,
  &quot;input&quot;,
  &quot;link&quot;,
  &quot;meta&quot;,
  &quot;param&quot;,
  &quot;source&quot;,
  &quot;track&quot;,
  &quot;wbr&quot;,
])</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L29-L44" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-tsx">// tsconfig.json or vite.config.ts
{
  &quot;compilerOptions&quot;: {
    &quot;jsx&quot;: &quot;react-jsx&quot;,
    &quot;jsxImportSource&quot;: &quot;@ox-content/vite-plugin&quot;
  }
}

// MyComponent.tsx
export function Hero({ title }: { title: string }) {
  return (
    &lt;section class=&quot;hero&quot;&gt;
      &lt;h1&gt;{title}&lt;/h1&gt;
    &lt;/section&gt;
  );
}</code></pre>
</div>
</div>
  </div>
</details>

<details id="when" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">when(condition: boolean, content: JSXNode): JSXNode</code><span class="ox-api-entry__description">Conditionally renders content.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns JSXNode</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Conditionally renders content.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function when(condition: boolean, content: JSXNode): JSXNode</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L274-L276" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">condition</code>
    <code class="ox-api-entry__param-type">boolean</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">content</code>
    <code class="ox-api-entry__param-type"><a href="#jsxnode">JSXNode</a></code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="#jsxnode">JSXNode</a></code>
  
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-tsx">{when(isLoggedIn, &lt;UserMenu /&gt;)}</code></pre>
</div>
</div>
  </div>
</details>

