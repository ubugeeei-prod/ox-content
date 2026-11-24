import { copyFile } from "node:fs/promises";
await copyFile(
  new URL("../src/virtual.d.ts", import.meta.url),
  new URL("../dist/virtual.d.ts", import.meta.url),
);
