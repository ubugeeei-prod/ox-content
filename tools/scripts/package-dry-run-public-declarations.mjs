import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, symlinkSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { publicDeclarationEntries } from "./public-declaration-contracts.mjs";
import { publicDeclarationValueUsage } from "./package-dry-run-html-host.mjs";
import { solidHtmlHostRegistryFixture } from "./package-dry-run-solid-html-host.mjs";

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
  const entries = publicDeclarationEntries.filter(
    (candidate) => candidate.packageName === pkg.name,
  );
  if (entries.length === 0) return;

  for (const entry of entries) {
    for (const extension of ["mts", "cts"]) {
      const declaration = readPackedFile(
        tarball,
        `dist/${publicDeclarationBase(entry)}.d.${extension}`,
      );
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

  if (pkg.name === "@ox-content/vite-plugin-solid") {
    checkSolidHtmlHostRegistryConsumer({ tarball, packDir, failures, packedPackages });
  }
}

function checkSolidHtmlHostRegistryConsumer({ tarball, packDir, failures, packedPackages }) {
  const entry = {
    packageName: "@ox-content/vite-plugin-solid",
    packageDir: "npm/vite-plugin-ox-content-solid",
    specifier: "@ox-content/vite-plugin-solid",
    packedDependencies: ["@ox-content/vite-plugin", "@ox-content/islands"],
    runtimeLinks: ["@types/node", "vite", "@solidjs/vite-plugin", "@solidjs/web", "solid-js"],
  };
  const consumerRoot = prepareConsumer({ tarball, entry, packDir, packedPackages });
  for (const dependency of [
    "@ox-content/napi",
    "@standard-schema/spec",
    "minimatch",
    "yaml",
    "glob",
    "rehype-parse",
    "rehype-stringify",
    "unified",
  ]) {
    linkPackageDependency(consumerRoot, "npm/vite-plugin-ox-content", dependency);
  }
  writeFileSync(join(consumerRoot, "package.json"), JSON.stringify({ type: "module" }));
  writeFileSync(join(consumerRoot, "registry-fixture.ts"), solidHtmlHostRegistryFixture());
  writeFileSync(join(consumerRoot, "tsconfig.json"), tsconfig("bundler", ["registry-fixture.ts"]));

  const result = spawnSync(tscBin, ["-p", join(consumerRoot, "tsconfig.json")], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    failures.push(`${entry.specifier} registry consumer failed:\n${result.stdout}${result.stderr}`);
  }
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
    `dist/${publicDeclarationBase(entry)}.d.mts`,
    `dist/${publicDeclarationBase(entry)}.d.cts`,
  ]) {
    const content = readPackedFile(tarball, path);
    for (const token of entry.browserOnlyForbidden) {
      if (content.includes(token)) {
        failures.push(`${entry.packageName} ${path} contains browser-forbidden ${token}`);
      }
    }
  }
}

function publicDeclarationBase(entry) {
  return entry.publicDistBase ?? entry.distBase;
}

function checkTypeConsumer({ tarball, entry, packDir, failures, packedPackages, mode }) {
  const consumerRoot = prepareConsumer({ tarball, entry, packDir, packedPackages });
  writeFileSync(join(consumerRoot, "package.json"), JSON.stringify({ type: "module" }));
  writeFileSync(join(consumerRoot, "esm-fixture.ts"), esmFixture(entry));
  writeFileSync(join(consumerRoot, "cjs-fixture.cts"), cjsFixture(entry));
  writeFileSync(
    join(consumerRoot, "tsconfig.json"),
    tsconfig(mode, undefined, entry.typeConsumerSkipLibCheck ?? true),
  );

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
  const statement =
    mode === "import"
      ? `const mod = await import(${JSON.stringify(entry.specifier)});`
      : `const mod = require(${JSON.stringify(entry.specifier)});`;
  const source = [statement, runtimeAssertions("mod", entry)].join("\n");
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
    publicDeclarationValueUsage(entry, ""),
  ].join("\n");
}

function cjsFixture(entry) {
  const namespace = entry.distBase.replace(/-([a-z])/g, (_, char) => char.toUpperCase());
  return [
    `import ${namespace} = require(${JSON.stringify(entry.specifier)});`,
    cjsTypeAliases(entry.types, entry.specifier),
    publicDeclarationValueUsage(entry, `${namespace}.`),
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

function runtimeAssertions(namespace, entry) {
  return entry.values
    .map(
      (name) =>
        `if (typeof ${namespace}.${name} === "undefined") throw new Error("Missing ${name}");`,
    )
    .join("\n");
}

function tsconfig(mode, files, skipLibCheck = true) {
  const compilerOptions =
    mode === "bundler"
      ? {
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "Bundler",
          lib: ["ES2022", "DOM"],
          strict: true,
          skipLibCheck,
          noEmit: true,
        }
      : {
          target: "ES2022",
          module: mode === "nodenext" ? "NodeNext" : "Node16",
          moduleResolution: mode === "nodenext" ? "NodeNext" : "Node16",
          lib: ["ES2022", "DOM"],
          strict: true,
          skipLibCheck,
          noEmit: true,
        };

  return JSON.stringify(
    {
      compilerOptions,
      files:
        files ?? (mode === "bundler" ? ["esm-fixture.ts"] : ["esm-fixture.ts", "cjs-fixture.cts"]),
    },
    null,
    2,
  );
}

function escapeRegExp(value) {
  return value.replace(/[\\^$.*+?()[\]{}|]/g, "\\$&");
}
