import type { IncomingMessage, ServerResponse } from "node:http";
import * as path from "node:path";
import { afterEach, describe, expect, it, vi } from "vite-plus/test";
import type { ViteDevServer } from "vite";
import type { CollectionAssetManifest } from "./collection-assets";
import { createCustomHostCollectionAssetsDevController } from "./custom-host-collection-assets";
import type { OxContentCustomHostBaseContext } from "./custom-host-types";

afterEach(() => vi.restoreAllMocks());

function setup(load: () => Promise<CollectionAssetManifest>) {
  const root = process.cwd();
  const onReplanned = vi.fn();
  const controller = createCustomHostCollectionAssetsDevController({
    server: { watcher: { add: vi.fn(), unwatch: vi.fn() } } as unknown as ViteDevServer,
    context: { root, base: "/" } as OxContentCustomHostBaseContext,
    options: {
      manifest: load,
      watch: [{ path: "content", kind: "directory" }],
      ownedPrefixes: ["/assets/content"],
    },
    beforeReplan: vi.fn(),
    onReplanned,
  })!;
  return { controller, onReplanned, changed: path.join(root, "content", "asset.txt") };
}

describe("custom host collection asset snapshot reads", () => {
  it("serves the published manifest immediately while a refresh is pending", async () => {
    const first: CollectionAssetManifest = { assets: [] };
    const second: CollectionAssetManifest = { assets: [] };
    const refresh = Promise.withResolvers<CollectionAssetManifest>();
    const load = vi.fn().mockResolvedValueOnce(first).mockReturnValueOnce(refresh.promise);
    const { controller, changed, onReplanned } = setup(load);
    await expect(controller.manifest()).resolves.toBe(first);
    controller.invalidate(changed);

    const received: CollectionAssetManifest[] = [];
    const read = controller.manifest().then((value) => received.push(value!));
    try {
      await Promise.resolve();
      expect(received).toEqual([first]);
    } finally {
      refresh.resolve(second);
      await read;
      await vi.waitFor(() => expect(onReplanned).toHaveBeenCalledOnce());
      controller.close();
    }
    expect(load).toHaveBeenCalledTimes(2);
  });

  it("retains a failed refresh for manifest and middleware reads without retrying per request", async () => {
    const first: CollectionAssetManifest = { assets: [] };
    const recovered: CollectionAssetManifest = { assets: [] };
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const load = vi
      .fn()
      .mockResolvedValueOnce(first)
      .mockRejectedValueOnce(new Error("refresh failed"))
      .mockResolvedValueOnce(recovered);
    const { controller, changed, onReplanned } = setup(load);
    await controller.manifest();
    controller.invalidate(changed);
    await vi.waitFor(() => expect(warn).toHaveBeenCalledOnce());

    for (let request = 0; request < 3; request += 1) {
      await expect(controller.manifest()).resolves.toBe(first);
      const next = vi.fn();
      await controller.middleware(
        { method: "GET", url: "/page" } as IncomingMessage,
        {} as ServerResponse,
        next,
      );
      expect(next).toHaveBeenCalledExactlyOnceWith();
    }
    expect(load).toHaveBeenCalledTimes(2);
    expect(onReplanned).not.toHaveBeenCalled();

    controller.invalidate(changed);
    await vi.waitFor(() => expect(onReplanned).toHaveBeenCalledOnce());
    await expect(controller.manifest()).resolves.toBe(recovered);
    expect(load).toHaveBeenCalledTimes(3);
    controller.close();
  });

  it("propagates cold-load failures and retries when no successful snapshot exists", async () => {
    const recovered: CollectionAssetManifest = { assets: [] };
    const load = vi
      .fn()
      .mockRejectedValueOnce(new Error("cold failure"))
      .mockResolvedValue(recovered);
    const { controller } = setup(load);
    await expect(controller.manifest()).rejects.toThrow("cold failure");
    await expect(controller.manifest()).resolves.toBe(recovered);
    expect(load).toHaveBeenCalledTimes(2);
    controller.close();
  });
});
