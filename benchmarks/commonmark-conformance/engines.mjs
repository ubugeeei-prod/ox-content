/**
 * Engine registry for the CommonMark conformance sweep.
 *
 * Kept separate from the runner because the configuration choices below are the
 * part of this sweep most open to dispute, and they belong together where they
 * can be reviewed without the harness around them.
 *
 * Each engine runs in the most CommonMark-faithful configuration it exposes,
 * not in the configuration the speed benchmark uses. The published column
 * answers "does this engine implement CommonMark", so judging an engine by a
 * default preset that deliberately turns on GFM extensions — or that escapes
 * the raw HTML the spec requires passing through — would measure the preset
 * instead of the engine. Where an engine exposes no such mode, that is noted at
 * its entry and it is scored as it ships.
 */

/** Loads an optional engine the same defensive way the speed benchmark does. */
async function optional(name, load) {
  try {
    return await load();
  } catch {
    console.warn(`${name} not available, skipping`);
    return null;
  }
}

const MIZCHI_COMMONMARK_OPTIONS = { autolink: false, tagfilter: false };

/** @returns {Promise<Array<[string, (markdown: string) => string]>>} */
export async function collectJsRenderers() {
  const renderers = [];

  const napi = await optional("@ox-content/napi", () => import("@ox-content/napi"));
  if (napi) {
    renderers.push(["@ox-content/napi", (input) => napi.parseAndRender(input).html]);
  }

  // marked exposes no CommonMark preset; scored as it ships.
  const marked = await optional("marked", () => import("marked"));
  if (marked) renderers.push(["marked", (input) => marked.marked(input)]);

  // md4w's DEFAULT flags enable tables, strikethrough, task lists, and
  // permissive autolinks. `parseFlags: 0` is md4c's plain CommonMark mode.
  const md4w = await optional("md4w", () => import("md4w"));
  if (md4w) {
    await md4w.init();
    renderers.push(["md4w (md4c)", (input) => md4w.mdToHtml(input, { parseFlags: 0 })]);
  }

  // md4x wraps the same engine but exposes no flag to turn its GFM extensions
  // off, so it is scored with them on.
  const md4x = await optional("md4x", () => import("md4x/napi"));
  if (md4x) renderers.push(["md4x (napi)", (input) => md4x.renderToHtml(input)]);

  // Same engine again through its wasm build, so the two runtimes' scores
  // can be compared directly (and against @ox-content/wasm below).
  const md4xWasm = await optional("md4x (wasm)", async () => {
    const mod = await import("md4x/wasm");
    await mod.init();
    return mod;
  });
  if (md4xWasm) renderers.push(["md4x (wasm)", (input) => md4xWasm.renderToHtml(input)]);

  // Our wasm-pack output is generated (`vp run build:wasm`), not checked in;
  // score it when the pkg exists so the wasm runtime gets conformance
  // coverage too.
  const oxWasm = await optional("@ox-content/wasm", async () => {
    const { readFileSync } = await import("node:fs");
    const pkgDir = new URL("../../crates/ox_content_wasm/pkg/", import.meta.url);
    const mod = await import(new URL("ox_content_wasm.js", pkgDir).href);
    await mod.default({
      module_or_path: readFileSync(new URL("ox_content_wasm_bg.wasm", pkgDir)),
    });
    return mod;
  });
  if (oxWasm) {
    const probe = oxWasm.parseAndRender("# probe");
    const render =
      probe instanceof Map
        ? (input) => oxWasm.parseAndRender(input).get("html")
        : (input) => oxWasm.parseAndRender(input).html;
    renderers.push(["@ox-content/wasm", render]);
  }

  // markdown-it's default preset is not its CommonMark mode; the 'commonmark'
  // preset is what its own docs point at for spec compliance.
  const MarkdownIt = await optional(
    "markdown-it",
    async () => (await import("markdown-it")).default,
  );
  if (MarkdownIt) {
    const md = new MarkdownIt("commonmark");
    renderers.push(["markdown-it", (input) => md.render(input)]);
  }

  const MarkdownItTs = await optional(
    "markdown-it-ts",
    async () => (await import("markdown-it-ts")).default,
  );
  if (MarkdownItTs) {
    const mdTs = MarkdownItTs("commonmark");
    renderers.push(["markdown-it-ts", (input) => mdTs.render(input)]);
  }

  // micromark escapes raw HTML and non-http protocols unless asked not to.
  // CommonMark requires passing both through.
  const micromark = await optional("micromark", () => import("micromark"));
  if (micromark) {
    renderers.push([
      "micromark",
      (input) =>
        micromark.micromark(input, {
          allowDangerousHtml: true,
          allowDangerousProtocol: true,
        }),
    ]);
  }

  // remark-html sanitizes by default, which drops the raw HTML the spec keeps.
  const remark = await optional("remark", async () => {
    const { unified } = await import("unified");
    const remarkParse = (await import("remark-parse")).default;
    const remarkHtml = (await import("remark-html")).default;
    return unified().use(remarkParse).use(remarkHtml, { sanitize: false });
  });
  if (remark) renderers.push(["remark", (input) => String(remark.processSync(input))]);

  // satteri returns `{ html, frontmatter }` rather than a string.
  const satteri = await optional("satteri", () => import("satteri"));
  if (satteri) renderers.push(["satteri", (input) => satteri.markdownToHtml(input).html]);

  // @mizchi/markdown exposes CommonMark switches to disable its default
  // autolink and tagfilter extensions. Score both package runtimes with the
  // same strict options.
  const mizchi = await optional("@mizchi/markdown", () => import("@mizchi/markdown"));
  if (mizchi) {
    renderers.push([
      "@mizchi/markdown (js)",
      (input) => mizchi.toHtml(input, MIZCHI_COMMONMARK_OPTIONS),
    ]);
  }

  const mizchiWasm = await optional("@mizchi/markdown/wasm", () => import("@mizchi/markdown/wasm"));
  if (mizchiWasm) {
    renderers.push([
      "@mizchi/markdown (wasm)",
      (input) => mizchiWasm.toHtml(input, MIZCHI_COMMONMARK_OPTIONS),
    ]);
  }

  // @tanstack/markdown exposes no CommonMark mode; scored as it ships.
  const tanstack = await optional("@tanstack/markdown", async () => {
    const [{ parseMarkdown }, { renderHtml }] = await Promise.all([
      import("@tanstack/markdown/parser"),
      import("@tanstack/markdown/html"),
    ]);
    return { parseMarkdown, renderHtml };
  });
  if (tanstack) {
    renderers.push([
      "@tanstack/markdown",
      (input) => tanstack.renderHtml(tanstack.parseMarkdown(input)),
    ]);
  }

  return renderers;
}
