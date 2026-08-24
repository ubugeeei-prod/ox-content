# jsx-runtime.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts)**

> 1 documented symbol. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>1</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>examples</span>
</span>
</div>

## Reference

<details id="jsx-runtime" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">jsx-runtime</code><span class="ox-api-entry__description">Automatic-runtime entry point for the static HTML JSX transform. jsxImportSource makes the compiler emit import { jsx }…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Automatic-runtime entry point for the static HTML JSX transform.</p>
<p><code>jsxImportSource</code> makes the compiler emit <code>import { jsx } from &quot;&lt;source&gt;/jsx-runtime&quot;</code>, so this subpath has to exist as its own module — the same symbols on the package&#39;s main entry cannot satisfy it.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/jsx-runtime.ts#L1-L15" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<div class="ox-api-entry__prose">
<pre><code class="language-json">{ &quot;compilerOptions&quot;: { &quot;jsx&quot;: &quot;react-jsx&quot;, &quot;jsxImportSource&quot;: &quot;@ox-content/vite-plugin&quot; } }</code></pre>
<p>The implementation lives in ./jsx-html; this file only re-exports what the transform reaches for. Named exports only, matching React&#39;s runtime shape.</p>
</div>
</div>
</div>
  </div>
</details>
