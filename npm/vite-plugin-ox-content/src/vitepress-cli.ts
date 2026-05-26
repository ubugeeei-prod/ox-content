#!/usr/bin/env node
import {
  createVitePressMigrationCliRuntime,
  runVitePressMigrationCli,
} from "./vitepress-cli-runtime";

const runtime = createVitePressMigrationCliRuntime();

runVitePressMigrationCli(runtime).catch(async (error) => {
  await runtime.writeStderr(`${error instanceof Error ? error.message : String(error)}\n`);
  runtime.setExitCode(1);
});
