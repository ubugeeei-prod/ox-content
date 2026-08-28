import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const publicValues = ["TWEET_COPY_RESET_MS", "initTweetCards", "initTwitterCards"];
const publicTypes = ["TweetCardsRoot", "InitTweetCardsOptions"];
const tscBin = join("node_modules", ".bin", process.platform === "win32" ? "tsc.cmd" : "tsc");

export function checkTwitterClientDeclarations({
  pkg,
  tarball,
  packDir,
  failures,
  readPackedFile,
}) {
  for (const extension of ["mts", "cts"]) {
    const declaration = readPackedFile(tarball, `dist/twitter-client.d.${extension}`);
    for (const name of [...publicValues, ...publicTypes]) {
      if (!new RegExp(`\\b${name}\\b`).test(declaration)) {
        failures.push(`${pkg.name} twitter-client.d.${extension} is missing ${name}`);
      }
    }
  }

  for (const mode of ["bundler", "nodenext", "node16"]) {
    checkTwitterClientConsumer({ pkg, tarball, packDir, failures, mode });
  }
}

function checkTwitterClientConsumer({ pkg, tarball, packDir, failures, mode }) {
  const consumerRoot = mkdtempSync(join(packDir, `twitter-client-${mode}-`));
  const packageRoot = join(consumerRoot, "node_modules", "@ox-content", "vite-plugin");
  mkdirSync(packageRoot, { recursive: true });

  const extract = spawnSync("tar", ["-xzf", tarball, "-C", packageRoot, "--strip-components=1"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (extract.error) {
    throw extract.error;
  }
  if (extract.status !== 0) {
    throw new Error(extract.stderr || `Failed to extract ${pkg.name} into ${consumerRoot}`);
  }

  writeFileSync(join(consumerRoot, "package.json"), JSON.stringify({ type: "module" }));
  writeFileSync(join(consumerRoot, "esm-fixture.ts"), esmFixture());
  writeFileSync(join(consumerRoot, "cjs-fixture.cts"), cjsFixture());
  writeFileSync(join(consumerRoot, "tsconfig.json"), tsconfig(mode));

  const result = spawnSync(tscBin, ["-p", join(consumerRoot, "tsconfig.json")], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    failures.push(
      `${pkg.name} twitter/client ${mode} consumer failed:\n${result.stdout}${result.stderr}`,
    );
  }
}

function esmFixture() {
  return [
    `import { ${publicValues.join(", ")} } from "@ox-content/vite-plugin/twitter/client";`,
    `import type { ${publicTypes.join(", ")} } from "@ox-content/vite-plugin/twitter/client";`,
    "",
    "declare const root: TweetCardsRoot;",
    "const options: InitTweetCardsOptions = { copiedMs: TWEET_COPY_RESET_MS };",
    "initTweetCards(root, options);",
    "initTwitterCards(root, options);",
  ].join("\n");
}

function cjsFixture() {
  return [
    `import twitterClient = require("@ox-content/vite-plugin/twitter/client");`,
    `type TweetCardsRoot = import("@ox-content/vite-plugin/twitter/client").TweetCardsRoot;`,
    `type InitTweetCardsOptions = import("@ox-content/vite-plugin/twitter/client").InitTweetCardsOptions;`,
    "",
    "declare const root: TweetCardsRoot;",
    "const options: InitTweetCardsOptions = { copiedMs: twitterClient.TWEET_COPY_RESET_MS };",
    "twitterClient.initTweetCards(root, options);",
    "twitterClient.initTwitterCards(root, options);",
  ].join("\n");
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
