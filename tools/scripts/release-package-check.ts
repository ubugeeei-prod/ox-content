import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { CARGO_PUBLISH_PACKAGES, NPM_PACKAGES } from "./release-targets.ts";

function run(command: string, args: string[], cwd = process.cwd()): void {
  execFileSync(command, args, { cwd, stdio: "inherit" });
}

if (process.argv.includes("--crates")) {
  // Package together so Cargo resolves not-yet-published internal versions locally.
  // Compilation verifies the actual archives, including include_str!/include_bytes! assets.
  run("cargo", ["package", ...CARGO_PUBLISH_PACKAGES.flatMap((name) => ["-p", name])]);
} else {
  const packages = [
    ...NPM_PACKAGES.filter((dir) => dir !== "npm/vscode-ox-content"),
    "crates/ox_content_wasm/pkg",
    ...readdirSync("binding-packages").map((dir) => `binding-packages/${dir}`),
  ];
  const destination = mkdtempSync(join(tmpdir(), "release-pack-"));
  try {
    for (const directory of packages) {
      const pkg = JSON.parse(readFileSync(join(directory, "package.json"), "utf8"));
      // A staged first version reserves the name before its public packument exists.
      const visibility = JSON.parse(
        execFileSync(
          "npm",
          ["access", "get", "status", pkg.name, "--json", "--registry=https://registry.npmjs.org"],
          { encoding: "utf8" },
        ),
      );
      if (visibility[pkg.name] !== "public") {
        throw new Error(`${pkg.name} must be registered as a public npm package before release.`);
      }
      run("vp", [
        "exec",
        "--",
        "pnpm",
        "--dir",
        directory,
        "pack",
        "--pack-destination",
        destination,
      ]);
    }
  } finally {
    rmSync(destination, { recursive: true, force: true });
  }
}
