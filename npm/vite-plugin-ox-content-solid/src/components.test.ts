import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vite-plus/test";
import { resolveComponentsGlob } from "./components";

describe("resolveComponentsGlob", () => {
  let root = "";

  beforeAll(async () => {
    root = await fs.promises.mkdtemp(path.join(os.tmpdir(), "ox-content-solid-glob-"));
    await fs.promises.mkdir(path.join(root, "src/components/nested"), { recursive: true });
    await fs.promises.writeFile(path.join(root, "src/components/alert.tsx"), "");
    await fs.promises.writeFile(path.join(root, "src/components/nested/info-box.tsx"), "");
    await fs.promises.writeFile(path.join(root, "src/components/notes.md"), "");
  });

  afterAll(async () => {
    await fs.promises.rm(root, { recursive: true, force: true });
  });

  it("matches files in a single directory", async () => {
    expect(await resolveComponentsGlob("./src/components/*.tsx", root)).toEqual({
      Alert: "./src/components/alert.tsx",
    });
  });

  it("keeps the extension filter for recursive patterns", async () => {
    // `**/*.tsx` splits into more than two segments, so the suffix has to be
    // read from the last one; otherwise every file (including notes.md) matches.
    expect(await resolveComponentsGlob("./src/**/*.tsx", root)).toEqual({
      Alert: "./src/components/alert.tsx",
      InfoBox: "./src/components/nested/info-box.tsx",
    });
  });
});
