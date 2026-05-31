# math.ts

> 3 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>3</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>8</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>returns</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>examples</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="clamp" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">clamp(value: number, min: number, max: number): number</code><span class="ox-api-entry__description">Clamps a number between a minimum and maximum value.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span><span class="ox-api-badge">returns number</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Clamps a number between a minimum and maximum value.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function clamp(value: number, min: number, max: number): number</code></pre>
</div>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">value</code>
    <code class="ox-api-entry__param-type">number</code>
  </div>
  <p class="ox-api-entry__param-description">The value to clamp</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">min</code>
    <code class="ox-api-entry__param-type">number</code>
  </div>
  <p class="ox-api-entry__param-description">The minimum allowed value</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">max</code>
    <code class="ox-api-entry__param-type">number</code>
  </div>
  <p class="ox-api-entry__param-description">The maximum allowed value</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">number</code>
  <p class="ox-api-entry__return-description">The clamped value</p>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-ts">clamp(5, 0, 10) // =&gt; 5
clamp(-5, 0, 10) // =&gt; 0
clamp(15, 0, 10) // =&gt; 10</code></pre>
</div>
</div>
  </div>
</details>

<details id="lerp" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">lerp(start: number, end: number, t: number): number</code><span class="ox-api-entry__description">Linearly interpolates between two values.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span><span class="ox-api-badge">returns number</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Linearly interpolates between two values.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function lerp(start: number, end: number, t: number): number</code></pre>
</div>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">start</code>
    <code class="ox-api-entry__param-type">number</code>
  </div>
  <p class="ox-api-entry__param-description">The start value</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">end</code>
    <code class="ox-api-entry__param-type">number</code>
  </div>
  <p class="ox-api-entry__param-description">The end value</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">t</code>
    <code class="ox-api-entry__param-type">number</code>
  </div>
  <p class="ox-api-entry__param-description">The interpolation factor (0 to 1)</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">number</code>
  <p class="ox-api-entry__return-description">The interpolated value</p>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-ts">lerp(0, 100, 0.5) // =&gt; 50
lerp(0, 100, 0.25) // =&gt; 25</code></pre>
</div>
</div>
  </div>
</details>

<details id="round" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">round(value: number, decimals: number): number</code><span class="ox-api-entry__description">Rounds a number to a specified number of decimal places.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns number</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Rounds a number to a specified number of decimal places.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function round(value: number, decimals: number): number</code></pre>
</div>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">value</code>
    <code class="ox-api-entry__param-type">number</code>
  </div>
  <p class="ox-api-entry__param-description">The value to round</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">decimals</code>
    <code class="ox-api-entry__param-type">number</code>
  </div>
  <p class="ox-api-entry__param-description">The number of decimal places</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">number</code>
  <p class="ox-api-entry__return-description">The rounded value</p>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-ts">round(3.14159, 2) // =&gt; 3.14
round(3.14159, 4) // =&gt; 3.1416</code></pre>
</div>
</div>
  </div>
</details>

