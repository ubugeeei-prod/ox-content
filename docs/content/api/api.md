# api.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/github/api.ts)**

> 4 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>4</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>4</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>8</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>4</strong>
  <span>returns</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="fetchgithubsource" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">fetchGitHubSource(source: GitHubSourceRef, options: Required&lt;GitHubOptions&gt;): Promise&lt;GitHubSourceData | null&gt;</code><span class="ox-api-entry__description">Fetch source file data from GitHub API.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;GitHubSourceData | null&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Fetch source file data from GitHub API.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function fetchGitHubSource(source: GitHubSourceRef, options: Required&lt;GitHubOptions&gt;): Promise&lt;GitHubSourceData | null&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/github/api.ts#L79-L145" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">source</code>
    <code class="ox-api-entry__param-type">GitHubSourceRef</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">Required&lt;GitHubOptions&gt;</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;GitHubSourceData | null&gt;</code>
  
</div>
</div>
  </div>
</details>

<details id="fetchrepodata" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">fetchRepoData(repo: string, options: Required&lt;GitHubOptions&gt;): Promise&lt;GitHubRepoData | null&gt;</code><span class="ox-api-entry__description">Fetch repository data from GitHub API.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;GitHubRepoData | null&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Fetch repository data from GitHub API.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function fetchRepoData(repo: string, options: Required&lt;GitHubOptions&gt;): Promise&lt;GitHubRepoData | null&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/github/api.ts#L39-L74" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">repo</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">Required&lt;GitHubOptions&gt;</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;GitHubRepoData | null&gt;</code>
  
</div>
</div>
  </div>
</details>

<details id="prefetchgithubrepos" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">prefetchGitHubRepos(repos: string[], options?: GitHubOptions): Promise&lt;Map&lt;string, GitHubRepoData | null&gt;&gt;</code><span class="ox-api-entry__description">Pre-fetch all GitHub repos data.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;Map&lt;string, GitHubRepoData | null&gt;&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Pre-fetch all GitHub repos data.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function prefetchGitHubRepos(repos: string[], options?: GitHubOptions): Promise&lt;Map&lt;string, GitHubRepoData | null&gt;&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/github/api.ts#L150-L165" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">repos</code>
    <code class="ox-api-entry__param-type">string[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">GitHubOptions</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;Map&lt;string, GitHubRepoData | null&gt;&gt;</code>
  
</div>
</div>
  </div>
</details>

<details id="prefetchgithubsources" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">prefetchGitHubSources(sources: GitHubSourceRef[], options?: GitHubOptions): Promise&lt;Map&lt;string, GitHubSourceData | null&gt;&gt;</code><span class="ox-api-entry__description">Pre-fetch all GitHub source files.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;Map&lt;string, GitHubSourceData | null&gt;&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Pre-fetch all GitHub source files.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function prefetchGitHubSources(sources: GitHubSourceRef[], options?: GitHubOptions): Promise&lt;Map&lt;string, GitHubSourceData | null&gt;&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/github/api.ts#L170-L188" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">sources</code>
    <code class="ox-api-entry__param-type">GitHubSourceRef[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">GitHubOptions</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;Map&lt;string, GitHubSourceData | null&gt;&gt;</code>
  
</div>
</div>
  </div>
</details>
