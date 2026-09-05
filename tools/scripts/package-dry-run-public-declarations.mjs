import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, symlinkSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { publicDeclarationEntries } from "./public-declaration-contracts.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const tscBin = join("node_modules", ".bin", process.platform === "win32" ? "tsc.cmd" : "tsc");

export function checkPublicDeclarationExports({
  pkg,
  tarball,
  packDir,
  failures,
  readPackedFile,
  packedPackages,
}) {
  const entry = publicDeclarationEntries.find((candidate) => candidate.packageName === pkg.name);
  if (!entry) return;

  for (const extension of ["mts", "cts"]) {
    const declaration = readPackedFile(tarball, `dist/${entry.distBase}.d.${extension}`);
    checkDeclarationNames({ declaration, entry, extension, failures });
  }

  if (entry.browserOnlyForbidden) {
    checkBrowserOnlyGraph({ tarball, entry, failures, readPackedFile });
  }

  for (const mode of ["bundler", "nodenext", "node16"]) {
    checkTypeConsumer({ tarball, entry, packDir, failures, packedPackages, mode });
  }
  checkRuntimeConsumer({ tarball, entry, packDir, failures, packedPackages, mode: "import" });
  checkRuntimeConsumer({ tarball, entry, packDir, failures, packedPackages, mode: "require" });
}

function checkDeclarationNames({ declaration, entry, extension, failures }) {
  for (const name of [...entry.values, ...entry.types]) {
    if (!new RegExp(`\\b${escapeRegExp(name)}\\b`).test(declaration)) {
      failures.push(`${entry.packageName} ${entry.distBase}.d.${extension} is missing ${name}`);
    }
    if (new RegExp(`\\b${escapeRegExp(name)}\\s+as\\s+\\w+\\b`).test(declaration)) {
      failures.push(`${entry.packageName} ${entry.distBase}.d.${extension} aliases ${name}`);
    }
  }

  for (const name of entry.forbiddenValueExports ?? []) {
    if (new RegExp(`export\\s*\\{[^}]*\\b${escapeRegExp(name)}\\b[^}]*\\}`).test(declaration)) {
      failures.push(`${entry.packageName} ${entry.distBase}.d.${extension} exports ${name}`);
    }
  }
}

function checkBrowserOnlyGraph({ tarball, entry, failures, readPackedFile }) {
  for (const path of [
    `dist/${entry.distBase}.mjs`,
    `dist/${entry.distBase}.cjs`,
    `dist/${entry.distBase}.d.mts`,
    `dist/${entry.distBase}.d.cts`,
  ]) {
    const content = readPackedFile(tarball, path);
    for (const token of entry.browserOnlyForbidden) {
      if (content.includes(token)) {
        failures.push(`${entry.packageName} ${path} contains browser-forbidden ${token}`);
      }
    }
  }
}

function checkTypeConsumer({ tarball, entry, packDir, failures, packedPackages, mode }) {
  const consumerRoot = prepareConsumer({ tarball, entry, packDir, packedPackages });
  writeFileSync(join(consumerRoot, "package.json"), JSON.stringify({ type: "module" }));
  writeFileSync(join(consumerRoot, "esm-fixture.ts"), esmFixture(entry));
  writeFileSync(join(consumerRoot, "cjs-fixture.cts"), cjsFixture(entry));
  writeFileSync(join(consumerRoot, "tsconfig.json"), tsconfig(mode));

  const result = spawnSync(tscBin, ["-p", join(consumerRoot, "tsconfig.json")], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    failures.push(`${entry.specifier} ${mode} consumer failed:\n${result.stdout}${result.stderr}`);
  }
}

function checkRuntimeConsumer({ tarball, entry, packDir, failures, packedPackages, mode }) {
  const consumerRoot = prepareConsumer({ tarball, entry, packDir, packedPackages });
  const file = mode === "import" ? "runtime-fixture.mjs" : "runtime-fixture.cjs";
  const source =
    mode === "import"
      ? [
          `const mod = await import(${JSON.stringify(entry.specifier)});`,
          runtimeAssertions("mod", entry),
        ].join("\n")
      : [
          `const mod = require(${JSON.stringify(entry.specifier)});`,
          runtimeAssertions("mod", entry),
        ].join("\n");
  writeFileSync(join(consumerRoot, file), source);

  const result = spawnSync(process.execPath, [join(consumerRoot, file)], {
    cwd: consumerRoot,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    failures.push(`${entry.specifier} runtime ${mode} failed:\n${result.stdout}${result.stderr}`);
  }
}

function prepareConsumer({ tarball, entry, packDir, packedPackages }) {
  const consumerRoot = mkdirTempRoot(packDir, entry);
  extractPackage(tarball, packageRoot(consumerRoot, entry.packageName), entry.packageName);

  for (const dependency of entry.packedDependencies ?? []) {
    const packed = packedPackages.get(dependency);
    if (!packed) throw new Error(`${entry.packageName} needs packed dependency ${dependency}`);
    extractPackage(packed.tarball, packageRoot(consumerRoot, dependency), dependency);
  }

  for (const dependency of entry.runtimeLinks ?? []) {
    linkPackageDependency(consumerRoot, entry.packageDir, dependency);
  }

  return consumerRoot;
}

function mkdirTempRoot(packDir, entry) {
  const safeName = entry.packageName.replace(/[^a-z0-9]+/gi, "-").replace(/^-|-$/g, "");
  return mkdtempSync(join(packDir, `${safeName}-`));
}

function extractPackage(tarball, destination, label) {
  mkdirSync(destination, { recursive: true });
  const result = spawnSync("tar", ["-xzf", tarball, "-C", destination, "--strip-components=1"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(result.stderr || `Failed to extract ${label}`);
  }
}

function linkPackageDependency(consumerRoot, packageDir, dependency) {
  const source = resolve(root, packageDir, "node_modules", ...dependency.split("/"));
  if (!existsSync(source)) return;
  const target = packageRoot(consumerRoot, dependency);
  mkdirSync(dirname(target), { recursive: true });
  try {
    symlinkSync(source, target, "junction");
  } catch (error) {
    if (error?.code !== "EEXIST") throw error;
  }
}

function packageRoot(consumerRoot, name) {
  return join(consumerRoot, "node_modules", ...name.split("/"));
}

function esmFixture(entry) {
  return [
    `import { ${entry.values.join(", ")} } from ${JSON.stringify(entry.specifier)};`,
    `import type { ${entry.types.join(", ")} } from ${JSON.stringify(entry.specifier)};`,
    typeAliases(entry.types),
    valueUsage(entry, ""),
  ].join("\n");
}

function cjsFixture(entry) {
  const namespace = entry.distBase.replace(/-([a-z])/g, (_, char) => char.toUpperCase());
  return [
    `import ${namespace} = require(${JSON.stringify(entry.specifier)});`,
    cjsTypeAliases(entry.types, entry.specifier),
    valueUsage(entry, `${namespace}.`),
  ].join("\n");
}

function typeAliases(types, prefix = "") {
  return types.map((name) => `type Use${name} = ${typeReference(name, prefix)};`).join("\n");
}

function typeReference(name, prefix) {
  return name === "MaybePromise" ? `${prefix}${name}<string>` : `${prefix}${name}`;
}

function cjsTypeAliases(types, specifier) {
  return types
    .map((name) =>
      name === "MaybePromise"
        ? `type ${name}<T> = import(${JSON.stringify(specifier)}).${name}<T>;`
        : `type ${name} = import(${JSON.stringify(specifier)}).${name};`,
    )
    .join("\n");
}

function valueUsage(entry, prefix) {
  if (entry.distBase === "custom-host") {
    return [
      "declare const customOptions: OxContentCustomHostOptions;",
      `const customPlugin = ${prefix}createOxContentCustomHostPlugin(customOptions);`,
      `const customOxOptions = ${prefix}customHostOxContentOptions();`,
      "void customPlugin;",
      "void customOxOptions;",
    ].join("\n");
  }

  return [
    `const hydrate = ${prefix}createSolidHtmlHostLazyHydrate({ modules: {}, render: () => {} });`,
    `${prefix}readSolidHtmlHostSlot({ dataset: {}, innerHTML: "" });`,
    `${prefix}initSolidHtmlHost({ initIslands: () => undefined, modules: {}, render: () => {} });`,
    "void hydrate;",
  ].join("\n");
}

function runtimeAssertions(namespace, entry) {
  return entry.values
    .map(
      (name) =>
        `if (typeof ${namespace}.${name} === "undefined") throw new Error("Missing ${name}");`,
    )
    .join("\n");
}

function tsconfig(mode) {
  const compilerOptions =
    mode === "bundler"
      ? {
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "Bundler",
          lib: ["ES2022", "DOM"],
          strict: true,
          skipLibCheck: true,
          noEmit: true,
        }
      : {
          target: "ES2022",
          module: mode === "nodenext" ? "NodeNext" : "Node16",
          moduleResolution: mode === "nodenext" ? "NodeNext" : "Node16",
          lib: ["ES2022", "DOM"],
          strict: true,
          skipLibCheck: true,
          noEmit: true,
        };

  return JSON.stringify(
    {
      compilerOptions,
      files: mode === "bundler" ? ["esm-fixture.ts"] : ["esm-fixture.ts", "cjs-fixture.cts"],
    },
    null,
    2,
  );
}

function escapeRegExp(value) {
  return value.replace(/[\\^$.*+?()[\]{}|]/g, "\\$&");
}
