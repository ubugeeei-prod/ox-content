import { spawnSync } from "node:child_process";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { delimiter, dirname, join, relative, resolve, sep } from "node:path";
import { describe, expect, it } from "vite-plus/test";
import {
  packageBuildConcurrencyEnvName,
  packageBuildConcurrencyFlag,
} from "../../scripts/package-build-concurrency";

const script = resolve(".github/scripts/run-pr-benchmark.mjs");

describe("run-pr-benchmark", () => {
  it("writes skipped reports without requiring a workspace checkout", () => {
    const dir = mkdtempSync(join(tmpdir(), "ox-pr-benchmark-"));
    const runtimeJson = join(dir, "runtime.json");
    const bundleJson = join(dir, "bundle.json");

    const result = spawnSync(
      process.execPath,
      [
        script,
        "--skip-runtime",
        "--skip-bundle",
        "--runtime-json",
        runtimeJson,
        "--bundle-json",
        bundleJson,
      ],
      {
        encoding: "utf8",
        env: { ...process.env, GITHUB_WORKSPACE: "" },
      },
    );

    expect(result.status).toBe(0);
    expect(JSON.parse(readFileSync(runtimeJson, "utf8"))).toMatchObject({
      name: "Parse/Render Speed Benchmark",
      skipped: true,
      sizes: {},
    });
    expect(JSON.parse(readFileSync(bundleJson, "utf8"))).toMatchObject({
      name: "Bundle Size Benchmark",
      skipped: true,
      results: [],
    });
  });

  it("keeps package benchmark builds at the default fanout locally", () => {
    expect(packageBuildConcurrencyFlag({})).toBe("");
  });

  it("lets CI raise package benchmark build fanout", () => {
    expect(packageBuildConcurrencyFlag({ [packageBuildConcurrencyEnvName]: "12" })).toBe(
      " --concurrency-limit 12",
    );
  });

  it("rejects invalid package benchmark build fanout", () => {
    expect(() => packageBuildConcurrencyFlag({ [packageBuildConcurrencyEnvName]: "0" })).toThrow(
      `${packageBuildConcurrencyEnvName} must be a positive integer`,
    );
  });

  it("runs copied benchmark scripts from the checkout's legacy benchmark workspace", () => {
    if (process.platform === "win32") {
      return;
    }

    const root = mkdtempSync(join(tmpdir(), "ox-pr-benchmark-layout-"));
    const source = join(root, "source");
    const checkout = join(root, "checkout");
    const bin = join(root, "bin");
    const runtimeJson = join(root, "runtime.json");
    const bundleJson = join(root, "bundle.json");
    const artifactsJson = join(root, "artifacts.json");

    mkdirSync(bin, { recursive: true });
    writeExecutable(join(bin, "vp"), "#!/bin/sh\nexit 0\n");
    writeExecutable(join(bin, "node"), `#!/bin/sh\nexec '${escapeShell(process.execPath)}' "$@"\n`);

    writeFile(
      join(source, "tools/benchmarks/bundle-size/parse-benchmark.mjs"),
      writesInvokedScriptJson,
    );
    writeFile(join(source, "tools/benchmarks/bundle-size/measure.mjs"), writesInvokedScriptJson);
    writeFile(
      join(source, "tools/benchmarks/bundle-size/measure-artifacts.mjs"),
      writesInvokedScriptJson,
    );
    writeFile(join(source, "tools/benchmarks/bundle-size/package.json"), '{"type":"module"}\n');

    for (const file of [
      "mizchi-markdown-native.mjs",
      "mizchi-markdown-native-template.mjs",
      "bundle-size/parse-benchmark-bun.mjs",
      "native-competitors/Cargo.toml",
      "native-competitors/Cargo.lock",
      "native-competitors/src/bench.rs",
      "native-competitors/src/cli.rs",
      "native-competitors/src/conformance.rs",
      "native-competitors/src/json.rs",
      "native-competitors/src/main.rs",
    ]) {
      writeFile(join(source, "tools/benchmarks", file), "");
    }

    writeFile(join(checkout, "benchmarks/bundle-size/package.json"), '{"type":"module"}\n');

    const result = spawnSync(
      process.execPath,
      [
        script,
        "--source",
        source,
        "--runtime-json",
        runtimeJson,
        "--bundle-json",
        bundleJson,
        "--artifacts-json",
        artifactsJson,
      ],
      {
        cwd: checkout,
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${bin}${delimiter}${process.env.PATH ?? ""}`,
        },
      },
    );

    expect(result.status, result.stderr || result.stdout).toBe(0);
    expectBenchmarkScript(runtimeJson, join("benchmarks", "bundle-size", "parse-benchmark.mjs"));
    expectBenchmarkScript(bundleJson, join("benchmarks", "bundle-size", "measure.mjs"));
    expectBenchmarkScript(
      artifactsJson,
      join("benchmarks", "bundle-size", "measure-artifacts.mjs"),
    );
  });

  it("keeps benchmark app workspace links relative to the moved benchmark root", () => {
    const lockfile = readFileSync(resolve("pnpm-lock.yaml"), "utf8");

    expectWorkspaceLink(
      lockfile,
      "tools/benchmarks/bundle-size",
      "@ox-content/napi",
      "crates/ox_content_napi",
    );
    expectWorkspaceLink(
      lockfile,
      "tools/benchmarks/bundle-size/apps/ox-content",
      "@ox-content/vite-plugin",
      "npm/vite-plugin-ox-content",
    );
    expectWorkspaceLink(
      lockfile,
      "tools/benchmarks/bundle-size/apps/ox-content-bare",
      "@ox-content/vite-plugin",
      "npm/vite-plugin-ox-content",
    );
    expectWorkspaceLink(
      lockfile,
      "tools/benchmarks/bundle-size/apps/ox-content-vue",
      "@ox-content/vite-plugin",
      "npm/vite-plugin-ox-content",
    );
    expectWorkspaceLink(
      lockfile,
      "tools/benchmarks/bundle-size/apps/ox-content-vue",
      "@ox-content/vite-plugin-vue",
      "npm/vite-plugin-ox-content-vue",
    );
    expectWorkspaceLink(
      lockfile,
      "tools/benchmarks/commonmark-conformance",
      "@ox-content/napi",
      "crates/ox_content_napi",
    );
    expectWorkspaceLink(
      lockfile,
      "tools/benchmarks/scale",
      "@ox-content/vite-plugin",
      "npm/vite-plugin-ox-content",
    );
  });
});

const writesInvokedScriptJson = `import { writeFileSync } from "node:fs";
const jsonIndex = process.argv.indexOf("--json");
writeFileSync(process.argv[jsonIndex + 1], JSON.stringify({ script: process.argv[1] }) + "\\n");
`;

const writeFile = (path: string, content: string) => {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content);
};

const writeExecutable = (path: string, content: string) => {
  writeFile(path, content);
  chmodSync(path, 0o755);
};

const escapeShell = (value: string) => value.replaceAll("'", "'\\''");

const expectBenchmarkScript = (path: string, expectedSuffix: string) => {
  const { script } = JSON.parse(readFileSync(path, "utf8")) as { script: string };
  expect(script).toContain(expectedSuffix);
  expect(script).not.toContain(join("tools", "benchmarks"));
};

const expectWorkspaceLink = (
  lockfile: string,
  importer: string,
  dependency: string,
  target: string,
) => {
  const importerBlock = importerLockfileBlock(lockfile, importer);
  const expected = relative(importer, target).split(sep).join("/");
  expect(importerBlock).toContain(
    `      '${dependency}':\n        specifier: workspace:*\n        version: link:${expected}`,
  );
};

const importerLockfileBlock = (lockfile: string, importer: string) => {
  const start = lockfile.indexOf(`  ${importer}:\n`);
  expect(start).toBeGreaterThanOrEqual(0);

  const rest = lockfile.slice(start + 1);
  const next = rest.search(/\n  \S[^:\n]*:\n/);
  return next === -1 ? rest : rest.slice(0, next);
};
