import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

export function requirePublicNpmRegistration(name: string): void {
  // Staging reserves a package before its first public packument exists.
  const visibility = JSON.parse(
    execFileSync(
      "npm",
      ["access", "get", "status", name, "--json", "--registry=https://registry.npmjs.org"],
      { encoding: "utf8" },
    ),
  );
  if (visibility[name] !== "public") {
    throw new Error(`${name} must be registered as a public npm package before release.`);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const directories = process.argv.slice(2);
  if (!directories.length) throw new Error("Usage: npm-registry-check.ts <package-directory>...");
  for (const directory of directories) {
    const pkg = JSON.parse(readFileSync(join(directory, "package.json"), "utf8"));
    requirePublicNpmRegistration(pkg.name);
  }
  console.log("Every package is registered as public on npm.");
}
