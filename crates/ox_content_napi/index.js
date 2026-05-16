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
module.exports.parseAndRender = binding.parseAndRender;
module.exports.parseAndRenderAsync = binding.parseAndRenderAsync;
module.exports.lintMarkdown = binding.lintMarkdown;
module.exports.lintMarkdownDocuments = binding.lintMarkdownDocuments;
module.exports.render = binding.render;
module.exports.transform = binding.transform;
module.exports.transformAsync = binding.transformAsync;
module.exports.version = binding.version;
module.exports.extractFileDocs = binding.extractFileDocs;
module.exports.extractFileDocEntries = binding.extractFileDocEntries;
module.exports.generateOgImageSvg = binding.generateOgImageSvg;
module.exports.buildSearchIndex = binding.buildSearchIndex;
module.exports.searchIndex = binding.searchIndex;
module.exports.extractSearchContent = binding.extractSearchContent;
module.exports.generateSsgHtml = binding.generateSsgHtml;
module.exports.transformMermaid = binding.transformMermaid;
module.exports.loadDictionaries = binding.loadDictionaries;
module.exports.loadDictionariesFlat = binding.loadDictionariesFlat;
module.exports.generateI18nModule = binding.generateI18nModule;
module.exports.validateMf2 = binding.validateMf2;
module.exports.checkI18n = binding.checkI18n;
module.exports.extractTranslationKeys = binding.extractTranslationKeys;
