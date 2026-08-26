import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

export async function writeClientAsset(outDir: string, source: string | undefined): Promise<void> {
  if (!source) {
    return;
  }
  await mkdir(outDir, { recursive: true });
  await writeFile(path.join(outDir, "ox-code-play.js"), source);
}
