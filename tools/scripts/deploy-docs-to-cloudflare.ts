import { buildDocsForCloudflare } from "./build-docs-for-cloudflare.ts";
import { forwardCliArgs, runCf } from "./cloudflare-cli.ts";

await buildDocsForCloudflare();
runCf(["deploy", "--prebuilt", "--mode", "production", ...forwardCliArgs(process.argv.slice(2))]);
