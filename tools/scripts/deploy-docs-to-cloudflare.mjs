import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { prepareCloudflareBuildOutput } from "../deploy/build-output.mjs";
import { forwardCliArgs, run, runCf } from "./cloudflare-cli.mjs";
import { prepareDocsDeployment } from "./prepare-docs-deployment.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

run("vp", ["run", "build"]);
prepareDocsDeployment(root);
await prepareCloudflareBuildOutput(root);
runCf(["deploy", "--prebuilt", "--mode", "production", ...forwardCliArgs(process.argv.slice(2))]);
