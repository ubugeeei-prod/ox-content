import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { run, runWrangler } from "./cloudflare-cli.mjs";
import { prepareDocsDeployment } from "./prepare-docs-deployment.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

run("vp", ["run", "build"]);
prepareDocsDeployment(root);
runWrangler(["deploy", ...process.argv.slice(2)]);
