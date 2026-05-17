type NapiModule = typeof import("@ox-content/napi");

function getDefaultExport(value: unknown): object | undefined {
  if (!value || typeof value !== "object" || !("default" in value)) {
    return undefined;
  }

  const defaultExport = value.default;
  return defaultExport && typeof defaultExport === "object" ? defaultExport : undefined;
}

function normalizeNapiModule(mod: NapiModule): NapiModule {
  const defaultExport = getDefaultExport(mod);
  return defaultExport
    ? ({
        ...defaultExport,
        ...mod,
      } as NapiModule)
    : mod;
}

export async function importNapiModule(): Promise<NapiModule> {
  return normalizeNapiModule((await import("@ox-content/napi")) as NapiModule);
}

let syncNapiModule: NapiModule | null | undefined;

export function importNapiModuleSync(): NapiModule {
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
    const mod = require("@ox-content/napi") as NapiModule;
    syncNapiModule = normalizeNapiModule(mod);
    return syncNapiModule;
  } catch {
    syncNapiModule = null;
    throw new Error(
      "[ox-content] @ox-content/napi is required. Please ensure the NAPI module is built.",
    );
  }
}
