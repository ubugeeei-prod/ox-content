import { buildDocsForCloudflare } from "./build-docs-for-cloudflare.mjs";
import { forwardCliArgs, runCf } from "./cloudflare-cli.mjs";

await buildDocsForCloudflare();
runCf(["deploy", "--prebuilt", "--mode", "production", ...forwardCliArgs(process.argv.slice(2))]);
