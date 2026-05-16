export async function importNapiModule(): Promise<typeof import("@ox-content/napi")> {
  const mod = (await import("@ox-content/napi")) as typeof import("@ox-content/napi") & {
    default?: Partial<typeof import("@ox-content/napi")>;
  };

  if (mod.default && typeof mod.default === "object") {
    return {
      ...mod.default,
      ...mod,
    };
  }

  return mod;
}

let syncNapiModule: typeof import("@ox-content/napi") | null | undefined;

export function importNapiModuleSync(): typeof import("@ox-content/napi") {
  if (syncNapiModule) {
    return syncNapiModule;
  }

  if (syncNapiModule === null) {
    throw new Error(
      "[ox-content] @ox-content/napi is required. Please ensure the NAPI module is built.",
    );
  }

  try {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const mod = require("@ox-content/napi") as typeof import("@ox-content/napi") & {
      default?: Partial<typeof import("@ox-content/napi")>;
    };
    syncNapiModule =
      mod.default && typeof mod.default === "object"
        ? ({
            ...mod.default,
            ...mod,
          } as typeof import("@ox-content/napi"))
        : mod;
    return syncNapiModule;
  } catch {
    syncNapiModule = null;
    throw new Error(
      "[ox-content] @ox-content/napi is required. Please ensure the NAPI module is built.",
    );
  }
}
