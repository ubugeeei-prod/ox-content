import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vite-plus/test";
import { resolveComponentsGlob } from "./components";

describe("resolveComponentsGlob", () => {
  let root: string;

  beforeEach(async () => {
    root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-solid-components-"));
    await write("src/components/Alert.tsx", "export default () => null;");
    await write("src/components/nested/Counter.tsx", "export default () => null;");
    await write("src/components/styles.css", ".a {}");
    await write("src/components/nested/notes.md", "# notes");
  });

  afterEach(async () => {
    await fs.rm(root, { recursive: true, force: true });
  });

  it("matches one directory level for a segment wildcard", async () => {
    expect(await resolveComponentsGlob("src/components/*.tsx", root)).toEqual({
      Alert: "./src/components/Alert.tsx",
    });
  });

  it("matches nested directories for a recursive wildcard, ignoring other extensions", async () => {
    expect(await resolveComponentsGlob("src/components/**/*.tsx", root)).toEqual({
      Alert: "./src/components/Alert.tsx",
      Counter: "./src/components/nested/Counter.tsx",
    });
  });

  it("matches a wildcard directory segment without descending further", async () => {
    await write("src/components/nested/deep/Deep.tsx", "export default () => null;");

    expect(await resolveComponentsGlob("src/components/*/Counter.tsx", root)).toEqual({
      Counter: "./src/components/nested/Counter.tsx",
    });
  });

  it("matches a single-character wildcard in the file name", async () => {
    await write("src/components/Tab1.tsx", "export default () => null;");
    await write("src/components/Tab12.tsx", "export default () => null;");

    expect(await resolveComponentsGlob("src/components/Tab?.tsx", root)).toEqual({
      Tab1: "./src/components/Tab1.tsx",
    });
  });

  it("returns an explicit map unchanged", async () => {
    const map = { Alert: "./src/components/Alert.tsx" };
    expect(await resolveComponentsGlob(map, root)).toBe(map);
  });

  async function write(relativePath: string, contents: string): Promise<void> {
    const fullPath = path.join(root, relativePath);
    await fs.mkdir(path.dirname(fullPath), { recursive: true });
    await fs.writeFile(fullPath, contents);
  }
});
