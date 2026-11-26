import { resolve } from "node:path";
import {
  cleanBuildOutputDir,
  getWorkerAssetsDir,
  readBuildOutput,
  writeAssets,
  writeRootConfig,
  writeWorkerConfig,
} from "@cloudflare/build-output-utils";
import config from "./cloudflare.config.ts";

const deployRoot = (repositoryRoot: string): string => resolve(repositoryRoot, "tools/deploy");

export const cloudflareAssetsDir = (repositoryRoot: string): string =>
  getWorkerAssetsDir(deployRoot(repositoryRoot));

export async function prepareCloudflareBuildOutput(repositoryRoot: string): Promise<void> {
  const root = deployRoot(repositoryRoot);
  await cleanBuildOutputDir(root);
  await writeAssets({ root, sourceDirectory: resolve(repositoryRoot, "dist") });
  await writeWorkerConfig({ root, config: config.worker });
  await writeRootConfig({ root, buildContext: { isPreview: false, mode: "production" } });
  await readBuildOutput(root);
}
