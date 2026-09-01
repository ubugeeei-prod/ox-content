#!/usr/bin/env node

import { main } from "./oxct-runtime.mjs";

try {
  main(process.argv.slice(2));
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
