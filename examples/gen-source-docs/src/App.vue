<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import type { DocEntry, ExtractedDocs, GeneratedDocsData } from "@ox-content/vite-plugin";

type ModuleView = ExtractedDocs & {
  slug: string;
  label: string;
  sourceFile: string;
  scopes: string[];
};

type VisibleModule = ModuleView & {
  visibleEntries: DocEntry[];
};

const docsUrl = new URL("../docs/api/docs.json", import.meta.url).href;

const docsData = ref<GeneratedDocsData | null>(null);
const loading = ref(true);
const loadError = ref<string | null>(null);
const query = ref("");
const selectedModule = ref<string | null>(null);

function parseScopedQuery(value: string) {
  const scopes: string[] = [];
  const terms: string[] = [];

  for (const part of value.trim().split(/\s+/).filter(Boolean)) {
    if (part.startsWith("@") && part.length > 1) {
      scopes.push(part.slice(1).toLowerCase());
    } else {
      terms.push(part);
    }
  }

  return {
    text: terms.join(" ").trim(),
    scopes: [...new Set(scopes)],
  };
}

function moduleSlug(file: string): string {
  return (
    file
      .split("/")
      .pop()
      ?.replace(/\.[^.]+$/, "")
      .toLowerCase() ?? file.toLowerCase()
  );
}

function moduleLabel(file: string): string {
  const slug = moduleSlug(file);
  return slug.charAt(0).toUpperCase() + slug.slice(1);
}

function normalizeSignature(signature?: string): string {
  if (!signature) {
    return "";
  }

  return signature
    .replace(/\s+/g, " ")
    .replace(/^export\s+/, "")
    .replace(/^async\s+function\s+/, "")
    .replace(/^function\s+/, "")
    .replace(/^class\s+/, "")
    .trim();
}

function compactText(value: string | undefined, maxLength: number = 118): string {
  if (!value) {
    return "";
  }

  const normalized = value.replace(/\s+/g, " ").trim();
  if (normalized.length <= maxLength) {
    return normalized;
  }

  return `${normalized.slice(0, maxLength - 1).trimEnd()}…`;
}

function entrySearchText(entry: DocEntry): string {
  return [
    entry.name,
    entry.kind,
    entry.description,
    entry.signature,
    ...(entry.params ?? []).flatMap((param) => [param.name, param.type, param.description]),
    entry.returns?.type,
    entry.returns?.description,
    ...(entry.examples ?? []),
    ...Object.entries(entry.tags ?? {}).flatMap(([name, value]) => [name, value]),
  ]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();
}

function matchesText(entry: DocEntry, text: string): boolean {
  if (!text) {
    return true;
  }

  return entrySearchText(entry).includes(text.toLowerCase());
}

function formatTimestamp(value: string | undefined): string {
  if (!value) {
    return "now";
  }

  return new Intl.DateTimeFormat("ja-JP", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function toggleScope(scope: string) {
  const parsed = parseScopedQuery(query.value);
  const scopes = new Set(parsed.scopes);

  if (scopes.has(scope)) {
    scopes.delete(scope);
  } else {
    scopes.add(scope);
  }

  query.value = [...scopes].map((item) => `@${item}`).join(" ");
  if (parsed.text) {
    query.value = query.value ? `${query.value} ${parsed.text}` : parsed.text;
  }
}

function shouldAutoOpen(module: VisibleModule, entry: DocEntry): boolean {
  const parsed = parseScopedQuery(query.value);
  return (
    Boolean(parsed.text) ||
    filteredModules.value.length === 1 ||
    selectedModule.value === module.slug
  );
}

const parsedQuery = computed(() => parseScopedQuery(query.value));

const modules = computed<ModuleView[]>(() =>
  (docsData.value?.modules ?? []).map((module) => {
    const slug = moduleSlug(module.file);

    return {
      ...module,
      slug,
      label: moduleLabel(module.file),
      sourceFile: module.file.split("/").pop() ?? module.file,
      scopes: ["api", slug],
    };
  }),
);

const totalSymbols = computed(() =>
  modules.value.reduce((count, module) => count + module.entries.length, 0),
);

const availableScopes = computed(() => {
  const scopes = new Set<string>(["api"]);

  for (const module of modules.value) {
    for (const scope of module.scopes) {
      scopes.add(scope);
    }
  }

  return [...scopes];
});

const filteredModules = computed<VisibleModule[]>(() => {
  const { scopes, text } = parsedQuery.value;
  const normalizedText = text.toLowerCase();

  return modules.value
    .map((module) => {
      const scopeMatched =
        scopes.length === 0 || scopes.some((scope) => module.scopes.includes(scope));

      if (!scopeMatched) {
        return null;
      }

      const visibleEntries = text
        ? module.entries.filter((entry) => matchesText(entry, normalizedText))
        : module.entries;

      if (
        visibleEntries.length === 0 &&
        text &&
        !module.label.toLowerCase().includes(normalizedText)
      ) {
        return null;
      }

      return {
        ...module,
        visibleEntries: visibleEntries.length > 0 ? visibleEntries : module.entries,
      };
    })
    .filter((module): module is VisibleModule => Boolean(module));
});

const visibleModules = computed(() => {
  if (!selectedModule.value) {
    return filteredModules.value;
  }

  const selected = filteredModules.value.find((module) => module.slug === selectedModule.value);
  return selected ? [selected] : filteredModules.value;
});

onMounted(async () => {
  try {
    const response = await fetch(docsUrl);

    if (!response.ok) {
      throw new Error(`failed to load docs.json (${response.status})`);
    }

    docsData.value = (await response.json()) as GeneratedDocsData;
  } catch (error) {
    loadError.value = error instanceof Error ? error.message : "failed to load generated docs";
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="app-shell">
    <header class="hero">
      <div class="hero-copy">
        <span class="eyebrow">Auto-generated Source Docs</span>
        <h1>cargo doc の視界と、accordion の気持ちよさを両立する。</h1>
        <p>
          一行で API surface を俯瞰しながら、必要なところだけ深掘りする docs viewer。
          <code>@api clamp</code> のような prefix 付き検索もそのまま試せます。
        </p>
      </div>

      <div class="hero-stats">
        <div class="stat-card">
          <span class="stat-label">Modules</span>
          <strong>{{ modules.length }}</strong>
        </div>
        <div class="stat-card">
          <span class="stat-label">Symbols</span>
          <strong>{{ totalSymbols }}</strong>
        </div>
        <div class="stat-card">
          <span class="stat-label">Generated</span>
          <strong>{{ formatTimestamp(docsData?.generatedAt) }}</strong>
        </div>
      </div>
    </header>

    <section class="command-surface">
      <label class="search-panel">
        <span class="search-label">Search with scopes</span>
        <input
          v-model="query"
          class="search-input"
          type="text"
          placeholder="Try @api clamp or @utils truncate"
        />
        <span class="search-help">Prefix with <code>@scope</code> to narrow the surface.</span>
      </label>

      <div class="scope-pills">
        <button
          v-for="scope in availableScopes"
          :key="scope"
          class="scope-pill"
          :class="{ active: parsedQuery.scopes.includes(scope) }"
          type="button"
          @click="toggleScope(scope)"
        >
          @{{ scope }}
        </button>
      </div>
    </section>

    <div class="workspace">
      <aside class="module-rail">
        <div class="rail-title">Modules</div>
        <button
          class="module-button"
          :class="{ active: selectedModule === null }"
          type="button"
          @click="selectedModule = null"
        >
          All modules
        </button>
        <button
          v-for="module in filteredModules"
          :key="module.slug"
          class="module-button"
          :class="{ active: selectedModule === module.slug }"
          type="button"
          @click="selectedModule = selectedModule === module.slug ? null : module.slug"
        >
          <span>{{ module.label }}</span>
          <small>{{ module.visibleEntries.length }}</small>
        </button>
      </aside>

      <main class="docs-stage">
        <section v-if="loading" class="empty-state">
          <strong>Loading generated docs…</strong>
          <p>docs.json を読み込みながら surface を組み立てています。</p>
        </section>

        <section v-else-if="loadError" class="empty-state error">
          <strong>Generated docs could not be loaded.</strong>
          <p>{{ loadError }}</p>
        </section>

        <section v-else-if="visibleModules.length === 0" class="empty-state">
          <strong>No symbols matched.</strong>
          <p>検索語を緩めるか、<code>@api</code> のような scope を切り替えてみてください。</p>
        </section>

        <template v-else>
          <article v-for="module in visibleModules" :key="module.slug" class="module-card">
            <header class="module-header">
              <div>
                <span class="module-kicker">@{{ module.scopes[0] }}</span>
                <h2>{{ module.label }}</h2>
              </div>

              <div class="module-meta">
                <code>{{ module.sourceFile }}</code>
                <span>{{ module.visibleEntries.length }} visible</span>
              </div>
            </header>

            <p class="module-summary">
              一行サマリーで API surface を眺めつつ、必要になった瞬間にだけ開ける構成です。
            </p>

            <details
              v-for="entry in module.visibleEntries"
              :key="`${module.slug}:${entry.name}`"
              class="entry-card"
              :open="shouldAutoOpen(module, entry)"
            >
              <summary>
                <span class="entry-kind">{{ entry.kind }}</span>
                <code class="entry-name">{{ entry.name }}</code>
                <code v-if="normalizeSignature(entry.signature)" class="entry-signature">
                  {{ normalizeSignature(entry.signature) }}
                </code>
                <span class="entry-description">{{ compactText(entry.description) }}</span>
              </summary>

              <div class="entry-body">
                <p v-if="entry.description" class="entry-copy">
                  {{ entry.description }}
                </p>

                <section v-if="entry.signature" class="entry-section">
                  <h3>Signature</h3>
                  <pre><code>{{ entry.signature }}</code></pre>
                </section>

                <section v-if="entry.params?.length" class="entry-section">
                  <h3>Parameters</h3>
                  <table>
                    <thead>
                      <tr>
                        <th>Name</th>
                        <th>Type</th>
                        <th>Description</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr v-for="param in entry.params" :key="`${entry.name}:${param.name}`">
                        <td>
                          <code>{{ param.name }}</code>
                        </td>
                        <td>
                          <code>{{ param.type }}</code>
                        </td>
                        <td>{{ param.description }}</td>
                      </tr>
                    </tbody>
                  </table>
                </section>

                <section v-if="entry.returns" class="entry-section">
                  <h3>Returns</h3>
                  <p>
                    <code>{{ entry.returns.type }}</code>
                    <span v-if="entry.returns.description"> - {{ entry.returns.description }}</span>
                  </p>
                </section>

                <section v-if="entry.examples?.length" class="entry-section">
                  <h3>Examples</h3>
                  <pre
                    v-for="(example, index) in entry.examples"
                    :key="`${entry.name}:example:${index}`"
                  >
<code>{{ example.replace(/^```\w*\n?/, "").replace(/\n?```$/, "") }}</code>
                  </pre>
                </section>

                <section v-if="entry.tags && Object.keys(entry.tags).length" class="entry-section">
                  <h3>Tags</h3>
                  <div class="tag-grid">
                    <span
                      v-for="(value, name) in entry.tags"
                      :key="`${entry.name}:${name}`"
                      class="tag-pill"
                    >
                      @{{ name }} {{ value }}
                    </span>
                  </div>
                </section>
              </div>
            </details>
          </article>
        </template>
      </main>
    </div>
  </div>
</template>

<style scoped>
:global(body) {
  margin: 0;
  min-height: 100vh;
  background:
    radial-gradient(circle at top left, rgba(230, 120, 40, 0.2), transparent 28%),
    radial-gradient(circle at top right, rgba(6, 79, 126, 0.18), transparent 32%),
    linear-gradient(180deg, #fffaf3 0%, #f4efe6 100%);
  color: #18222d;
  font-family: "IBM Plex Sans", "Avenir Next", "Segoe UI", sans-serif;
}

:global(*) {
  box-sizing: border-box;
}

.app-shell {
  max-width: 1400px;
  margin: 0 auto;
  padding: 32px 24px 64px;
}

.hero {
  display: grid;
  grid-template-columns: minmax(0, 1.8fr) minmax(280px, 0.9fr);
  gap: 24px;
  align-items: end;
  margin-bottom: 24px;
}

.hero-copy h1 {
  margin: 10px 0 16px;
  font-family: "Iowan Old Style", "Palatino Linotype", "Book Antiqua", serif;
  font-size: clamp(2.8rem, 5vw, 4.8rem);
  line-height: 0.94;
  letter-spacing: -0.05em;
}

.hero-copy p {
  max-width: 780px;
  margin: 0;
  font-size: 1.05rem;
  line-height: 1.75;
  color: #44505e;
}

.hero-copy code,
.module-meta code,
.entry-card code,
.search-help code,
.empty-state code {
  font-family: "IBM Plex Mono", "SFMono-Regular", monospace;
}

.eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.72);
  border: 1px solid rgba(24, 34, 45, 0.08);
  font-size: 0.78rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #8a4a0f;
}

.hero-stats {
  display: grid;
  gap: 14px;
}

.stat-card,
.command-surface,
.module-rail,
.module-card,
.empty-state {
  border: 1px solid rgba(24, 34, 45, 0.08);
  background: rgba(255, 255, 255, 0.72);
  backdrop-filter: blur(20px);
  box-shadow: 0 30px 80px rgba(88, 63, 28, 0.08);
}

.stat-card {
  padding: 18px 20px;
  border-radius: 22px;
}

.stat-label {
  display: block;
  margin-bottom: 10px;
  font-size: 0.76rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #7b8794;
}

.stat-card strong {
  font-size: 1.35rem;
  line-height: 1.2;
}

.command-surface {
  display: grid;
  gap: 18px;
  margin-bottom: 24px;
  padding: 20px;
  border-radius: 28px;
}

.search-panel {
  display: grid;
  gap: 8px;
}

.search-label {
  font-size: 0.82rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #7b8794;
}

.search-input {
  width: 100%;
  padding: 16px 18px;
  border: 1px solid rgba(24, 34, 45, 0.12);
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.85);
  color: #18222d;
  font-size: 1rem;
  outline: none;
  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    transform 0.2s ease;
}

.search-input:focus {
  border-color: #c85f19;
  box-shadow: 0 0 0 4px rgba(200, 95, 25, 0.12);
  transform: translateY(-1px);
}

.search-help {
  color: #5d6977;
  font-size: 0.92rem;
}

.scope-pills {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.scope-pill {
  padding: 9px 14px;
  border: 1px solid rgba(24, 34, 45, 0.1);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.78);
  color: #44505e;
  cursor: pointer;
  font-size: 0.92rem;
  transition:
    transform 0.2s ease,
    border-color 0.2s ease,
    color 0.2s ease,
    background 0.2s ease;
}

.scope-pill:hover,
.scope-pill.active {
  transform: translateY(-1px);
  border-color: rgba(200, 95, 25, 0.45);
  background: rgba(200, 95, 25, 0.12);
  color: #8a4a0f;
}

.workspace {
  display: grid;
  grid-template-columns: 260px minmax(0, 1fr);
  gap: 20px;
  align-items: start;
}

.module-rail {
  position: sticky;
  top: 24px;
  display: grid;
  gap: 10px;
  padding: 16px;
  border-radius: 24px;
}

.rail-title {
  font-size: 0.8rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #7b8794;
}

.module-button {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  padding: 12px 14px;
  border: 0;
  border-radius: 18px;
  background: transparent;
  color: #24313f;
  cursor: pointer;
  font-size: 0.95rem;
  text-align: left;
  transition:
    background 0.2s ease,
    transform 0.2s ease;
}

.module-button small {
  color: #7b8794;
}

.module-button:hover,
.module-button.active {
  background: rgba(24, 34, 45, 0.06);
  transform: translateY(-1px);
}

.docs-stage {
  display: grid;
  gap: 18px;
}

.module-card,
.empty-state {
  padding: 24px;
  border-radius: 30px;
}

.module-header {
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: 18px;
  margin-bottom: 14px;
}

.module-header h2 {
  margin: 8px 0 0;
  font-size: 2rem;
  line-height: 1.05;
  letter-spacing: -0.04em;
}

.module-kicker {
  display: inline-flex;
  align-items: center;
  padding: 7px 12px;
  border-radius: 999px;
  background: rgba(6, 79, 126, 0.1);
  color: #0b4a74;
  font-size: 0.78rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.module-meta {
  display: grid;
  justify-items: end;
  gap: 8px;
  color: #607080;
  font-size: 0.9rem;
}

.module-summary {
  margin: 0 0 18px;
  color: #4d5a69;
  line-height: 1.65;
}

.entry-card {
  margin-top: 14px;
  border: 1px solid rgba(24, 34, 45, 0.1);
  border-radius: 22px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.84), rgba(248, 242, 232, 0.7));
  overflow: hidden;
}

.entry-card summary {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  padding: 16px 18px;
  cursor: pointer;
  list-style: none;
}

.entry-card summary::-webkit-details-marker {
  display: none;
}

.entry-kind {
  display: inline-flex;
  align-items: center;
  padding: 6px 10px;
  border-radius: 999px;
  background: rgba(24, 34, 45, 0.08);
  color: #607080;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.entry-name {
  font-size: 1rem;
  font-weight: 700;
}

.entry-signature {
  flex: 1 1 360px;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: #8a4a0f;
}

.entry-description {
  flex: 1 1 100%;
  color: #61707f;
}

.entry-body {
  padding: 0 18px 18px;
}

.entry-copy {
  margin-top: 0;
  line-height: 1.75;
  color: #34404c;
  white-space: pre-wrap;
}

.entry-section + .entry-section {
  margin-top: 16px;
}

.entry-section h3 {
  margin: 0 0 10px;
  font-size: 0.84rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #7b8794;
}

.entry-section p {
  margin: 0;
  color: #34404c;
}

.entry-section pre {
  margin: 0;
  padding: 16px;
  overflow: auto;
  border-radius: 18px;
  background: #16202b;
  color: #eef3f9;
  line-height: 1.6;
}

.entry-section table {
  width: 100%;
  border-collapse: collapse;
  overflow: hidden;
  border-radius: 18px;
}

.entry-section th,
.entry-section td {
  padding: 12px 14px;
  border-bottom: 1px solid rgba(24, 34, 45, 0.08);
  text-align: left;
  vertical-align: top;
}

.entry-section th {
  background: rgba(24, 34, 45, 0.04);
  color: #44505e;
  font-size: 0.82rem;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.tag-grid {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.tag-pill {
  padding: 8px 12px;
  border-radius: 999px;
  background: rgba(6, 79, 126, 0.08);
  color: #0b4a74;
  font-size: 0.88rem;
}

.empty-state {
  text-align: center;
}

.empty-state strong {
  display: block;
  margin-bottom: 10px;
  font-size: 1.2rem;
}

.empty-state p {
  margin: 0;
  color: #5d6977;
}

.empty-state.error {
  background: rgba(180, 48, 48, 0.06);
}

@media (max-width: 1080px) {
  .hero,
  .workspace {
    grid-template-columns: 1fr;
  }

  .module-rail {
    position: static;
  }
}

@media (max-width: 720px) {
  .app-shell {
    padding: 20px 14px 48px;
  }

  .hero-copy h1 {
    font-size: 2.5rem;
  }

  .module-header,
  .entry-card summary {
    flex-direction: column;
    align-items: flex-start;
  }

  .entry-signature {
    width: 100%;
  }

  .entry-section th,
  .entry-section td {
    padding: 10px 12px;
  }
}
</style>
