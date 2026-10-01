import { cpSync, rmSync, statSync } from "node:fs";
import { resolve } from "node:path";

export function prepareDocsDeployment(root: string): void {
  const sources = [
    ["docs/dist/docs", ""],
    ["target/doc", "api"],
    ["examples/playground/dist", "playground"],
  ];
  // Validate every input before replacing a previous deployment bundle.
  for (const [source] of sources) {
    if (!statSync(resolve(root, source)).isDirectory()) {
      throw new Error(`Missing deployment directory: ${source}`);
    }
  }
  const destination = resolve(root, "dist");
  rmSync(destination, { recursive: true, force: true });
  for (const [source, subdirectory] of sources) {
    cpSync(resolve(root, source), resolve(destination, subdirectory), { recursive: true });
  }
}
