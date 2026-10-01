import { resolve } from "node:path";
import {
  cleanBuildOutputDir,
  readBuildOutput,
  writeAssets,
  writeRootConfig,
  writeWorkerConfig,
} from "@cloudflare/build-output-utils";
import config from "./cloudflare.config.ts";

export async function prepareCloudflareBuildOutput(repositoryRoot) {
  const root = resolve(repositoryRoot, "tools/deploy");
  await cleanBuildOutputDir(root);
  await writeAssets({ root, sourceDirectory: resolve(repositoryRoot, "dist") });
  await writeWorkerConfig({ root, config: config.worker });
  await writeRootConfig(root, undefined, { isPreview: false, mode: "production" });
  await readBuildOutput(root);
}
