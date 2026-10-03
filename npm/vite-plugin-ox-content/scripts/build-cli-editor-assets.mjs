import { cp, mkdir } from "node:fs/promises";
const target = new URL("../dist/editors/neovim/", import.meta.url);
await mkdir(target, { recursive: true });
await cp(new URL("../../../editors/neovim/", import.meta.url), target, { recursive: true });
