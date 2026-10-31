export function publicDeclarationValueUsage(entry, prefix) {
  if (entry.specifier === "@ox-content/vite-plugin/markdown") {
    return markdownValueUsage(prefix);
  }
  if (entry.distBase === "custom-host") {
    return customHostValueUsage(prefix);
  }
  if (entry.specifier === "@ox-content/vite-plugin/html-host") {
    return htmlHostValueUsage(prefix);
  }
  if (entry.specifier === "@ox-content/islands/html-host") {
    return genericHtmlHostClientValueUsage(prefix);
  }
  return htmlHostClientValueUsage(entry, prefix);
}

function markdownValueUsage(prefix) {
  return [
    'const markdownInput: CreateOxContentMarkdownRendererInput = { root: ".", oxContent: { embeds: false } };',
    `const markdownRenderer: OxContentCustomHostMarkdownRenderer = ${prefix}createOxContentMarkdownRenderer(markdownInput);`,
    "const markdownPage: Promise<OxContentCustomHostMarkdownResult<{ nodes: number }>> = markdownRenderer.render({",
    '  source: "# Guide",',
    '  documentPath: "content/guide.md",',
    "  renderHtml(context: OxContentCustomHostMarkdownRenderContext) {",
    "    const tree = context.parseHtml();",
    '    const kind: "root" = tree.type;',
    "    void kind;",
    "    return { html: context.html, metadata: { nodes: tree.children.length } };",
    "  },",
    "});",
    `const markdownProcessor: MarkdownProcessor = ${prefix}createMarkdownProcessor();`,
    'const markdownTransform: Promise<TransformResult> = markdownProcessor.render("# Guide", "guide.md");',
    `const markdownRendered: Promise<TransformResult> = ${prefix}renderMarkdown("# Guide", "guide.md");`,
    "void markdownPage;",
    "void markdownTransform;",
    "void markdownRendered;",
  ].join("\n");
}

function customHostValueUsage(prefix) {
  return [
    "declare const customOptions: OxContentCustomHostOptions;",
    "declare const customAssets: OxContentCustomHostAssetsContext;",
    `const customPlugin = ${prefix}createOxContentCustomHostPlugin(customOptions);`,
    `const customOxOptions = ${prefix}customHostOxContentOptions();`,
    `const customStyles = customAssets.stylesheets({ modules: ["/src/Island.ts"] });`,
    "void customAssets.collectionManifest();",
    "void customAssets.stylesheetContent({ stylesheets: customStyles.stylesheets });",
    'const customDependency: OxContentCustomHostDependency = { path: "content/guide", kind: "directory" };',
    'const customCollectionAssets: OxContentCustomHostCollectionAssetsOptions = { manifest: { assets: [] }, watch: [customDependency], ownedPrefixes: ["/assets/content"] };',
    "customAssets.document({ islandStyles: customStyles.stylesheets });",
    "declare const customContext: OxContentCustomHostRenderContext;",
    "const customMarkdown = customContext.markdown.render<{ clientModules: string[] }>({",
    '  source: "# Guide",',
    '  documentPath: "content/guide.mdx",',
    "  async renderHtml(ctx: OxContentCustomHostMarkdownRenderContext) {",
    "    const styles = ctx.assets.stylesheets({ modules: [] });",
    "    return { html: ctx.html, metadata: { clientModules: [] }, dependencies: styles.dependencies };",
    "  },",
    "});",
    "const customRoute: OxContentCustomHostRoute = {",
    '  path: "/guide",',
    '  inputPath: "content/guide.md",',
    '  lastUpdatedPaths: ["src/site-owner.ts", "content/guide"],',
    "  dependencies: [customDependency],",
    '  render: () => ({ html: "<h1>Guide</h1>", lastUpdatedPaths: ["src/guide.ts"] }),',
    "};",
    'const customResult: OxContentCustomHostRenderResult = { html: "<h1>Guide</h1>", lastUpdatedPaths: ["src/guide.ts"] };',
    "const customDeps: string[] = customStyles.dependencies;",
    "const customLastmodSources: readonly string[] | undefined = customRoute.lastUpdatedPaths;",
    "void customResult.lastUpdatedPaths;",
    "void customCollectionAssets;",
    "void customMarkdown;",
    "void customPlugin;",
    "void customOxOptions;",
    "void customDeps;",
    "void customLastmodSources;",
  ].join("\n");
}

function htmlHostValueUsage(prefix) {
  return [
    'const htmlAdapter: HtmlHostFrameworkAdapter = { renderComponent: () => "" };',
    `const htmlRender = ${prefix}renderHtmlHost({ html: "", documentPath: "content/page.mdx", loadModule: async () => ({}), adapter: htmlAdapter });`,
    `const htmlRenderer = ${prefix}createHtmlHostRenderer({ loadModule: async () => ({}), adapter: htmlAdapter });`,
    `const htmlHydrate = ${prefix}createHtmlHostHydrate({ components: {}, render: () => {} });`,
    `const htmlRegistry = ${prefix}createHtmlHostIslandRegistry({ entries: [{ name: "Probe", moduleId: "./Probe.ts" }] });`,
    `const htmlDocuments = ${prefix}createHtmlHostCollectionDocuments({});`,
    'declare const htmlMarkdownContext: RenderHtmlHostMarkdownInput["context"];',
    `const htmlMarkdown = ${prefix}renderHtmlHostMarkdown({ context: htmlMarkdownContext, renderIslands: htmlRenderer });`,
    `void ${prefix}HTML_HOST_MODULES_VIRTUAL_ID;`,
    `void ${prefix}HtmlHostRenderError;`,
    `void ${prefix}formatHtmlHostDiagnostics([]);`,
    `void ${prefix}isBareHtmlHostSpecifier("pkg");`,
    `void ${prefix}resolveHtmlHostCollectionDocuments({}, { root: ".", mode: "production", command: "build" });`,
    `void ${prefix}resolveHtmlHostComponentsGlob({}, ".");`,
    `void ${prefix}resolveHtmlHostDocumentPath("content/page.mdx", ".");`,
    `void ${prefix}resolveHtmlHostIslandRegistry({}, { root: ".", mode: "production", command: "build" });`,
    `void ${prefix}resolveHtmlHostWatchFile("content/page.mdx", ".");`,
    `void ${prefix}shouldInvalidateHtmlHostRegistry("content/page.mdx", ".", "content", [], []);`,
    `void ${prefix}toHtmlHostClientModuleId("./Probe.ts", ".");`,
    "void htmlRender;",
    "void htmlRenderer;",
    "void htmlHydrate;",
    "void htmlRegistry;",
    "void htmlDocuments;",
    "void htmlMarkdown;",
  ].join("\n");
}

function htmlHostClientValueUsage(entry, prefix) {
  const names = htmlHostClientNames(entry);
  return [
    `const hydrate = ${prefix}${names.createLazyHydrate}({ modules: {}, render: () => {} });`,
    `const domRenderer = ${prefix}${names.createDomRenderer}({ mode: "render" });`,
    `const domHydrate = ${prefix}${names.createLazyHydrate}({ modules: {}, mount: { mode: "render" } });`,
    `void ${prefix}${names.loadDomRuntime};`,
    `${prefix}${names.readSlot}({ dataset: {}, innerHTML: "" });`,
    `${prefix}${names.initHost}({ initIslands: () => undefined, modules: {}, render: () => {} });`,
    "void hydrate;",
    "void domRenderer;",
    "void domHydrate;",
  ].join("\n");
}

function genericHtmlHostClientValueUsage(prefix) {
  return [
    "declare const element: HTMLElement;",
    `const hydrate = ${prefix}createHtmlHostLazyHydrate({ modules: {}, render: () => {} });`,
    `const init = ${prefix}initHtmlHost({ initIslands: (run) => ({ run }), modules: {}, render: () => {} });`,
    `const handle: HtmlHostHydrationHandle = hydrate(element, {});`,
    `const error = ${prefix}createHtmlHostClientError("missing-module-id", element, {}, { frameworkName: "HTML host" });`,
    `${prefix}reportHtmlHostClientError({ onError: () => {} }, error, "html-host:error");`,
    `${prefix}readHtmlHostSlot({ dataset: {}, innerHTML: "" });`,
    `void ${prefix}HtmlHostClientHydrationError;`,
    "handle();",
    "handle.then(() => undefined);",
    "void init;",
  ].join("\n");
}

function htmlHostClientNames(entry) {
  const prefix = entry.packageName.endsWith("-svelte") ? "Svelte" : "Solid";
  return {
    initHost: `init${prefix}HtmlHost`,
    createDomRenderer: `create${prefix}HtmlHostDomRenderer`,
    createLazyHydrate: `create${prefix}HtmlHostLazyHydrate`,
    loadDomRuntime: `load${prefix}HtmlHostDomRuntime`,
    readSlot: `read${prefix}HtmlHostSlot`,
  };
}
