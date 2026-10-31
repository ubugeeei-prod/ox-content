import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import type { ResolvedConfig, ViteDevServer } from "vite";
import { createHostLoaderServer } from "./custom-host-loader";

const roots: string[] = [];
const servers: ViteDevServer[] = [];

afterEach(async () => {
  await Promise.all(servers.splice(0).map((server) => server.close()));
  await Promise.all(roots.splice(0).map((root) => fs.rm(root, { recursive: true, force: true })));
});

describe("custom host build loader", () => {
  it.each([{ port: 3000 }, { watch: { usePolling: true } }])(
    "loads modules without watching when consumer server options are %j",
    async (serverOptions) => {
      const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-host-loader-"));
      roots.push(root);
      const configFile = path.join(root, "vite.config.mjs");
      await fs.writeFile(configFile, `export default ${JSON.stringify({ server: serverOptions })}`);
      await fs.writeFile(path.join(root, "host.mjs"), "export const title = 'Loaded';");

      const server = await createHostLoaderServer({
        root,
        configFile,
        mode: "production",
        logLevel: "silent",
      } as ResolvedConfig);
      servers.push(server);

      expect(server.config.server.watch).toBeNull();
      expect(server.watcher.getWatched()).toEqual({});
      expect(await server.ssrLoadModule(path.join(root, "host.mjs"))).toMatchObject({
        title: "Loaded",
      });
      expect(server.watcher.getWatched()).toEqual({});
    },
  );
});
