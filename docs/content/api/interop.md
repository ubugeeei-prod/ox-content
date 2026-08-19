# interop.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/interop.ts)**

> 2 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>2</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>parameters</span>
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

<details id="interop" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">interop</code><span class="ox-api-entry__description">Interop helpers for loading ESM-only dependencies from the CommonJS build.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Interop helpers for loading ESM-only dependencies from the CommonJS build.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/interop.ts#L1-L3" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="interopdefault" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">interopDefault&lt;T&gt;(value: T): T</code><span class="ox-api-entry__description">Unwrap the default export of an ESM-only dependency. The CommonJS build (dist/i…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns T</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Unwrap the <code>default</code> export of an ESM-only dependency.</p>
<p>The CommonJS build (<code>dist/index.cjs</code>) loads pure-ESM packages such as <code>rehype-parse</code> and <code>rehype-stringify</code> through <code>require(esm)</code>. The bundler&#39;s interop helper then double-wraps their default export: what should be the plugin function arrives as the module namespace <code>{ default: fn }</code> instead of <code>fn</code> itself. Passing that object to <code>unified().use()</code> throws &quot;Expected usable value but received an empty preset&quot;.</p>
<p>Calling <code>interopDefault</code> on a default import unwraps one extra level of <code>default</code> nesting, so the same code works whether the dependency was loaded as native ESM (<code>.mjs</code>) or through the CommonJS interop (<code>.cjs</code>). See #452.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function interopDefault&lt;T&gt;(value: T): T</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/interop.ts#L19-L24" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">value</code>
    <code class="ox-api-entry__param-type">T</code>
  </div>

</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">T</code>

</div>
</div>
  </div>
</details>
