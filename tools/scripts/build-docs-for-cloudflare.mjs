import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { prepareCloudflareBuildOutput } from "../deploy/build-output.mjs";
import { run } from "./cloudflare-cli.mjs";
import { prepareDocsDeployment } from "./prepare-docs-deployment.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

export async function buildDocsForCloudflare() {
  run("vp", ["run", "build"]);
  prepareDocsDeployment(root);
  await prepareCloudflareBuildOutput(root);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await buildDocsForCloudflare();
}
