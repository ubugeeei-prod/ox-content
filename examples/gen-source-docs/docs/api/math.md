# math.ts

> 3 documented symbols. Skim the one-line surface first, then expand the accordions for details.

## Overview

- [`clamp`](#clamp) `function` `clamp(value: number, min: number, max: number): number` - Clamps a number between a minimum and maximum value.
- [`lerp`](#lerp) `function` `lerp(start: number, end: number, t: number): number` - Linearly interpolates between two values.
- [`round`](#round) `function` `round(value: number, decimals: number): number` - Rounds a number to a specified number of decimal places.

## Reference

<a id="clamp"></a>

<details class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">function</span><code class="ox-api-entry__name">clamp</code><code class="ox-api-entry__signature">clamp(value: number, min: number, max: number): number</code><span class="ox-api-entry__description">Clamps a number between a minimum and maximum value.</span></summary>
  <div class="ox-api-entry__body">
<p>Clamps a number between a minimum and maximum value.</p>
<div class="ox-api-entry__section">
<h4>Signature</h4>
<pre><code class="language-typescript">export function clamp(value: number, min: number, max: number): number</code></pre>
</div>
<div class="ox-api-entry__section">
<h4>Parameters</h4>
<table>
  <thead>
    <tr><th>Name</th><th>Type</th><th>Description</th></tr>
  </thead>
  <tbody>
<tr>
  <td><code>value</code></td>
  <td><code>number</code></td>
  <td>The value to clamp</td>
</tr>
<tr>
  <td><code>min</code></td>
  <td><code>number</code></td>
  <td>The minimum allowed value</td>
</tr>
<tr>
  <td><code>max</code></td>
  <td><code>number</code></td>
  <td>The maximum allowed value</td>
</tr>
  </tbody>
</table>
</div>
<div class="ox-api-entry__section">
<h4>Returns</h4>
<p><code>number</code> — The clamped value</p>
</div>
<div class="ox-api-entry__section">
<h4>Examples</h4>
<pre><code class="language-ts">clamp(5, 0, 10) // =&gt; 5
clamp(-5, 0, 10) // =&gt; 0
clamp(15, 0, 10) // =&gt; 10</code></pre>
</div>
  </div>
</details>

<a id="lerp"></a>

<details class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">function</span><code class="ox-api-entry__name">lerp</code><code class="ox-api-entry__signature">lerp(start: number, end: number, t: number): number</code><span class="ox-api-entry__description">Linearly interpolates between two values.</span></summary>
  <div class="ox-api-entry__body">
<p>Linearly interpolates between two values.</p>
<div class="ox-api-entry__section">
<h4>Signature</h4>
<pre><code class="language-typescript">export function lerp(start: number, end: number, t: number): number</code></pre>
</div>
<div class="ox-api-entry__section">
<h4>Parameters</h4>
<table>
  <thead>
    <tr><th>Name</th><th>Type</th><th>Description</th></tr>
  </thead>
  <tbody>
<tr>
  <td><code>start</code></td>
  <td><code>number</code></td>
  <td>The start value</td>
</tr>
<tr>
  <td><code>end</code></td>
  <td><code>number</code></td>
  <td>The end value</td>
</tr>
<tr>
  <td><code>t</code></td>
  <td><code>number</code></td>
  <td>The interpolation factor (0 to 1)</td>
</tr>
  </tbody>
</table>
</div>
<div class="ox-api-entry__section">
<h4>Returns</h4>
<p><code>number</code> — The interpolated value</p>
</div>
<div class="ox-api-entry__section">
<h4>Examples</h4>
<pre><code class="language-ts">lerp(0, 100, 0.5) // =&gt; 50
lerp(0, 100, 0.25) // =&gt; 25</code></pre>
</div>
  </div>
</details>

<a id="round"></a>

<details class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">function</span><code class="ox-api-entry__name">round</code><code class="ox-api-entry__signature">round(value: number, decimals: number): number</code><span class="ox-api-entry__description">Rounds a number to a specified number of decimal places.</span></summary>
  <div class="ox-api-entry__body">
<p>Rounds a number to a specified number of decimal places.</p>
<div class="ox-api-entry__section">
<h4>Signature</h4>
<pre><code class="language-typescript">export function round(value: number, decimals: number): number</code></pre>
</div>
<div class="ox-api-entry__section">
<h4>Parameters</h4>
<table>
  <thead>
    <tr><th>Name</th><th>Type</th><th>Description</th></tr>
  </thead>
  <tbody>
<tr>
  <td><code>value</code></td>
  <td><code>number</code></td>
  <td>The value to round</td>
</tr>
<tr>
  <td><code>decimals</code></td>
  <td><code>number</code></td>
  <td>The number of decimal places</td>
</tr>
  </tbody>
</table>
</div>
<div class="ox-api-entry__section">
<h4>Returns</h4>
<p><code>number</code> — The rounded value</p>
</div>
<div class="ox-api-entry__section">
<h4>Examples</h4>
<pre><code class="language-ts">round(3.14159, 2) // =&gt; 3.14
round(3.14159, 4) // =&gt; 3.1416</code></pre>
</div>
  </div>
</details>
