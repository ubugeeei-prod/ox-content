import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { delimiter, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const script = fileURLToPath(new URL("./retry-preview-publish.sh", import.meta.url));
const connectionError = "Failed to connect to server: TypeError: fetch failed";

for (const scenario of [
  { name: "succeeds without retrying", failures: 0, message: "", attempts: 1, status: 0 },
  {
    name: "recovers from a connection failure",
    failures: 1,
    message: connectionError,
    attempts: 2,
    status: 0,
  },
  {
    name: "stops after three connection failures",
    failures: 5,
    message: connectionError,
    attempts: 3,
    status: 42,
  },
  {
    name: "does not retry authorization failures",
    failures: 1,
    message: "Check failed (401): Unauthorized",
    attempts: 1,
    status: 42,
  },
  {
    name: "does not retry packaging failures",
    failures: 1,
    message: "pnpm pack failed",
    attempts: 1,
    status: 42,
  },
  {
    name: "does not retry failures during publication",
    failures: 1,
    message: "Publish failed: TypeError: fetch failed",
    attempts: 1,
    status: 42,
  },
]) {
  test(scenario.name, () => {
    const dir = mkdtempSync(join(tmpdir(), "ox-preview-retry-"));
    try {
      const counter = join(dir, "attempts");
      writeFileSync(counter, "0");
      // Skip real delays while exercising the shell and child process boundary.
      writeFileSync(join(dir, "sleep"), "#!/bin/sh\nexit 0\n", { mode: 0o755 });
      const result = spawnSync(
        "bash",
        [
          script,
          process.execPath,
          "--input-type=module",
          "-e",
          `
        import { readFileSync, writeFileSync } from "node:fs";
        const [counter, failures, message] = process.argv.slice(1);
        const attempt = Number(readFileSync(counter, "utf8")) + 1;
        writeFileSync(counter, String(attempt));
        console.log("attempt " + attempt);
        if (attempt <= Number(failures)) {
          console.error(message);
          process.exit(42);
        }
        console.log("published");
      `,
          counter,
          String(scenario.failures),
          scenario.message,
        ],
        {
          encoding: "utf8",
          env: { ...process.env, PATH: `${dir}${delimiter}${process.env.PATH}` },
        },
      );

      assert.equal(result.status, scenario.status, result.stderr);
      assert.equal(Number(readFileSync(counter, "utf8")), scenario.attempts);
      assert.match(result.stdout, /attempt 1/);
      if (scenario.status === 0) assert.match(result.stdout, /published/);
      if (scenario.failures > 0) assert.ok(result.stdout.includes(scenario.message));
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
}
