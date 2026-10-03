#!/usr/bin/env node
import { main } from "@ox-content/vite-plugin/cli";
try {
  await main(process.argv.slice(2));
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
