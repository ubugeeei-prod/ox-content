import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { prepareCloudflareBuildOutput } from "../deploy/build-output.ts";
import { run } from "./cloudflare-cli.ts";
import { prepareDocsDeployment } from "./prepare-docs-deployment.ts";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

export async function buildDocsForCloudflare(): Promise<void> {
  run("vp", ["run", "build"]);
  prepareDocsDeployment(root);
  await prepareCloudflareBuildOutput(root);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await buildDocsForCloudflare();
}
