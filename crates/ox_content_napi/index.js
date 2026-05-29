const { existsSync } = require("fs");
const path = require("path");

function loadBinding() {
  // 1. Try loading the local binary (napi build output)
  const napiOutput = path.join(__dirname, "ox-content.node");
  if (existsSync(napiOutput)) {
    return require(napiOutput);
  }

  // 1b. Legacy: index.node (napi-rs v2)
  const localBinary = path.join(__dirname, "index.node");
  if (existsSync(localBinary)) {
    return require(localBinary);
  }

  // 2. Try platform-specific binary in same directory (CI build artifact)
  const platform = process.platform;
  const arch = process.arch;

  const platforms = {
    "darwin-arm64": "ox-content.darwin-arm64.node",
    "darwin-x64": "ox-content.darwin-x64.node",
    "linux-x64-gnu": "ox-content.linux-x64-gnu.node",
    "linux-arm64-gnu": "ox-content.linux-arm64-gnu.node",
    "win32-x64-msvc": "ox-content.win32-x64-msvc.node",
  };

  let key;
  if (platform === "darwin") {
    key = `darwin-${arch}`;
  } else if (platform === "linux") {
    key = `linux-${arch}-gnu`;
  } else if (platform === "win32") {
    key = `win32-${arch}-msvc`;
  }

  const binaryName = platforms[key];
  if (binaryName) {
    const binaryPath = path.join(__dirname, binaryName);
    if (existsSync(binaryPath)) {
      return require(binaryPath);
    }
  }

  // 3. Try npm sub-packages (@ox-content/binding-darwin-arm64 etc.)
  const subPackages = {
    "darwin-arm64": "@ox-content/binding-darwin-arm64",
    "darwin-x64": "@ox-content/binding-darwin-x64",
    "linux-x64-gnu": "@ox-content/binding-linux-x64-gnu",
    "linux-arm64-gnu": "@ox-content/binding-linux-arm64-gnu",
    "win32-x64-msvc": "@ox-content/binding-win32-x64-msvc",
  };

  const subPackage = subPackages[key];
  if (subPackage) {
    try {
      return require(subPackage);
    } catch {}
  }

  throw new Error(
    `@ox-content/napi: No compatible binary found for ${platform}-${arch}. ` +
      `If you're working from the repository, run 'nix develop -c vp run build:napi' from the repository root.`,
  );
}

const binding = loadBinding();

// Export individual functions for ESM compatibility
module.exports = binding;
module.exports.parse = binding.parse;
module.exports.parseTransferRaw = binding.parseTransferRaw;
module.exports.parseMdastRaw = binding.parseMdastRaw;
module.exports.parseAndRender = binding.parseAndRender;
module.exports.parseAndRenderAsync = binding.parseAndRenderAsync;
module.exports.prepareSource = binding.prepareSource;
module.exports.prepareSourceRaw = binding.prepareSourceRaw;
module.exports.lintMarkdown = binding.lintMarkdown;
module.exports.lintMarkdownDocuments = binding.lintMarkdownDocuments;
module.exports.render = binding.render;
module.exports.transform = binding.transform;
module.exports.transformAsync = binding.transformAsync;
module.exports.transformMdastRaw = binding.transformMdastRaw;
module.exports.version = binding.version;
module.exports.extractFileDocs = binding.extractFileDocs;
module.exports.extractFileDocEntries = binding.extractFileDocEntries;
module.exports.buildExportGraph = binding.buildExportGraph;
module.exports.extractDocsFromDirectories = binding.extractDocsFromDirectories;
module.exports.extractDocsFromEntryPoints = binding.extractDocsFromEntryPoints;
module.exports.generateDocsNavMetadata = binding.generateDocsNavMetadata;
module.exports.generateDocsNavMetadataFromDocs = binding.generateDocsNavMetadataFromDocs;
module.exports.generateDocsNavCode = binding.generateDocsNavCode;
module.exports.collectDocsSourceFiles = binding.collectDocsSourceFiles;
module.exports.generateDocsDataJson = binding.generateDocsDataJson;
module.exports.generateDocsMarkdown = binding.generateDocsMarkdown;
module.exports.writeGeneratedDocs = binding.writeGeneratedDocs;
module.exports.mergeHighlightedCodeBlocks = binding.mergeHighlightedCodeBlocks;
module.exports.generateOgImageSvg = binding.generateOgImageSvg;
module.exports.buildSearchIndex = binding.buildSearchIndex;
module.exports.buildSearchIndexFromDirectory = binding.buildSearchIndexFromDirectory;
module.exports.writeSearchIndex = binding.writeSearchIndex;
module.exports.searchIndex = binding.searchIndex;
module.exports.extractSearchContent = binding.extractSearchContent;
module.exports.parseScopedSearchQuery = binding.parseScopedSearchQuery;
module.exports.getSearchDocumentScopes = binding.getSearchDocumentScopes;
module.exports.matchesSearchScopes = binding.matchesSearchScopes;
module.exports.generateSearchModule = binding.generateSearchModule;
module.exports.generateSearchModuleFromOptions = binding.generateSearchModuleFromOptions;
module.exports.collectSearchMarkdownFiles = binding.collectSearchMarkdownFiles;
module.exports.normalizeVitePressFrontmatter = binding.normalizeVitePressFrontmatter;
module.exports.generateSsgHtml = binding.generateSsgHtml;
module.exports.generateSsgBareHtml = binding.generateSsgBareHtml;
module.exports.getGitLastUpdated = binding.getGitLastUpdated;
module.exports.resolveSsgRoutePaths = binding.resolveSsgRoutePaths;
module.exports.getSsgOutputPath = binding.getSsgOutputPath;
module.exports.getSsgUrlPath = binding.getSsgUrlPath;
module.exports.getSsgHref = binding.getSsgHref;
module.exports.getSsgPageLocale = binding.getSsgPageLocale;
module.exports.extractSsgTitle = binding.extractSsgTitle;
module.exports.formatSsgTitle = binding.formatSsgTitle;
module.exports.buildSsgNavItems = binding.buildSsgNavItems;
module.exports.buildSsgThemeNavItems = binding.buildSsgThemeNavItems;
module.exports.resolveSsgNavigationGroups = binding.resolveSsgNavigationGroups;
module.exports.collectSsgMarkdownFiles = binding.collectSsgMarkdownFiles;
module.exports.externalizeSsgAssets = binding.externalizeSsgAssets;
module.exports.transformMermaid = binding.transformMermaid;
module.exports.loadDictionaries = binding.loadDictionaries;
module.exports.loadDictionariesFlat = binding.loadDictionariesFlat;
module.exports.generateI18nModule = binding.generateI18nModule;
module.exports.validateMf2 = binding.validateMf2;
module.exports.checkI18n = binding.checkI18n;
module.exports.checkI18nProject = binding.checkI18nProject;
module.exports.extractTranslationKeys = binding.extractTranslationKeys;
