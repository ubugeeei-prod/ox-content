# utils.ts

> 3 documented symbols. Skim the one-line surface first, then expand the accordions for details.

## Overview

- [`capitalize`](#capitalize) `function` `capitalize(str: string): string` - Capitalizes the first letter of a string.
- [`truncate`](#truncate) `function` `truncate(str: string, maxLength: number, suffix: string = "..."): string` - Truncates a string to a specified length.
- [`toKebabCase`](#tokebabcase) `function` `toKebabCase(str: string): string` - Converts a string to kebab-case.

## Reference

<a id="capitalize"></a>

<details class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">function</span><code class="ox-api-entry__name">capitalize</code><code class="ox-api-entry__signature">capitalize(str: string): string</code><span class="ox-api-entry__description">Capitalizes the first letter of a string.</span></summary>
  <div class="ox-api-entry__body">
<p>Capitalizes the first letter of a string.</p>
<div class="ox-api-entry__section">
<h4>Signature</h4>
<pre><code class="language-typescript">export function capitalize(str: string): string</code></pre>
</div>
<div class="ox-api-entry__section">
<h4>Parameters</h4>
<table>
  <thead>
    <tr><th>Name</th><th>Type</th><th>Description</th></tr>
  </thead>
  <tbody>
<tr>
  <td><code>str</code></td>
  <td><code>string</code></td>
  <td>The input string to capitalize</td>
</tr>
  </tbody>
</table>
</div>
<div class="ox-api-entry__section">
<h4>Returns</h4>
<p><code>string</code> — The string with the first letter capitalized</p>
</div>
<div class="ox-api-entry__section">
<h4>Examples</h4>
<pre><code class="language-ts">capitalize(&#39;hello&#39;) // =&gt; &#39;Hello&#39;
capitalize(&#39;WORLD&#39;) // =&gt; &#39;WORLD&#39;</code></pre>
</div>
  </div>
</details>

<a id="truncate"></a>

<details class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">function</span><code class="ox-api-entry__name">truncate</code><code class="ox-api-entry__signature">truncate(str: string, maxLength: number, suffix: string = &quot;...&quot;): string</code><span class="ox-api-entry__description">Truncates a string to a specified length.</span></summary>
  <div class="ox-api-entry__body">
<p>Truncates a string to a specified length.</p>
<div class="ox-api-entry__section">
<h4>Signature</h4>
<pre><code class="language-typescript">export function truncate(str: string, maxLength: number, suffix: string = &quot;...&quot;): string</code></pre>
</div>
<div class="ox-api-entry__section">
<h4>Parameters</h4>
<table>
  <thead>
    <tr><th>Name</th><th>Type</th><th>Description</th></tr>
  </thead>
  <tbody>
<tr>
  <td><code>str</code></td>
  <td><code>string</code></td>
  <td>The input string to truncate</td>
</tr>
<tr>
  <td><code>maxLength</code></td>
  <td><code>number</code></td>
  <td>Maximum length of the output string</td>
</tr>
<tr>
  <td><code>suffix</code></td>
  <td><code>string</code></td>
  <td>Suffix to append when truncated (default: &#39;...&#39;)</td>
</tr>
  </tbody>
</table>
</div>
<div class="ox-api-entry__section">
<h4>Returns</h4>
<p><code>string</code> — The truncated string</p>
</div>
<div class="ox-api-entry__section">
<h4>Examples</h4>
<pre><code class="language-ts">truncate(&#39;Hello World&#39;, 5) // =&gt; &#39;Hello...&#39;
truncate(&#39;Hi&#39;, 10) // =&gt; &#39;Hi&#39;</code></pre>
</div>
  </div>
</details>

<a id="tokebabcase"></a>

<details class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">function</span><code class="ox-api-entry__name">toKebabCase</code><code class="ox-api-entry__signature">toKebabCase(str: string): string</code><span class="ox-api-entry__description">Converts a string to kebab-case.</span></summary>
  <div class="ox-api-entry__body">
<p>Converts a string to kebab-case.</p>
<div class="ox-api-entry__section">
<h4>Signature</h4>
<pre><code class="language-typescript">export function toKebabCase(str: string): string</code></pre>
</div>
<div class="ox-api-entry__section">
<h4>Parameters</h4>
<table>
  <thead>
    <tr><th>Name</th><th>Type</th><th>Description</th></tr>
  </thead>
  <tbody>
<tr>
  <td><code>str</code></td>
  <td><code>string</code></td>
  <td>The input string to convert</td>
</tr>
  </tbody>
</table>
</div>
<div class="ox-api-entry__section">
<h4>Returns</h4>
<p><code>string</code> — The kebab-cased string</p>
</div>
<div class="ox-api-entry__section">
<h4>Examples</h4>
<pre><code class="language-ts">toKebabCase(&#39;helloWorld&#39;) // =&gt; &#39;hello-world&#39;
toKebabCase(&#39;HelloWorld&#39;) // =&gt; &#39;hello-world&#39;</code></pre>
</div>
  </div>
</details>
