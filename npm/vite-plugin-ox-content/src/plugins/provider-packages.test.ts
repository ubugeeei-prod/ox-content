import { afterEach, describe, expect, it } from "vite-plus/test";
import {
  clearProviderPackageCache,
  enrichProviderPackageEmbeds,
  parsePackageRegistryReference,
  type ProviderPackageFetch,
} from "./provider-packages";

const originalWarn = console.warn;

afterEach(() => {
  console.warn = originalWarn;
  clearProviderPackageCache();
});

describe("package registry provider cards", () => {
  it("parses supported registry URLs into stable metadata endpoints", () => {
    expect(
      parsePackageRegistryReference(
        "NpmPackage",
        "https://www.npmjs.com/package/@vitejs/plugin-vue/v/6.0.0",
      ),
    ).toEqual({
      provider: "npm",
      apiUrl: "https://registry.npmjs.org/%40vitejs%2Fplugin-vue",
      name: "@vitejs/plugin-vue",
      ecosystem: "npm",
      version: "6.0.0",
    });
    expect(
      parsePackageRegistryReference("CratesIo", "https://crates.io/crates/serde/1.0.0"),
    ).toMatchObject({
      provider: "crates.io",
      apiUrl: "https://crates.io/api/v1/crates/serde",
      name: "serde",
      version: "1.0.0",
    });
    expect(
      parsePackageRegistryReference("PyPI", "https://pypi.org/project/requests/2.32.0"),
    ).toMatchObject({
      provider: "pypi",
      apiUrl: "https://pypi.org/pypi/requests/2.32.0/json",
      name: "requests",
      version: "2.32.0",
    });
    expect(
      parsePackageRegistryReference(
        "DockerHub",
        "https://hub.docker.com/r/library/nginx/tags?name=mainline",
      ),
    ).toMatchObject({
      provider: "docker-hub",
      apiUrl: "https://hub.docker.com/v2/repositories/library/nginx",
      name: "library/nginx",
      version: "mainline",
    });
  });

  it("rejects unsafe or unsupported registry URLs", () => {
    expect(
      parsePackageRegistryReference("NpmPackage", "http://www.npmjs.com/package/vite"),
    ).toBeNull();
    expect(
      parsePackageRegistryReference("CratesIo", "https://user:pass@crates.io/crates/serde"),
    ).toBeNull();
    expect(parsePackageRegistryReference("PyPI", "https://pypi.org/user/requests")).toBeNull();
    expect(
      parsePackageRegistryReference("DockerHub", "https://hub.docker.com.evil/r/library/nginx"),
    ).toBeNull();
  });

  it("enriches package metadata once per provider and caches by registry reference", async () => {
    const requests: string[] = [];
    const fetchImpl: ProviderPackageFetch = async (input) => {
      const url = requestUrl(input);
      requests.push(url);
      return okJson(metadataFor(url));
    };
    const input = [
      '<NpmPackage url="https://www.npmjs.com/package/vite"></NpmPackage>',
      '<CratesIo url="https://crates.io/crates/serde"></CratesIo>',
      '<PyPI url="https://pypi.org/project/requests/2.32.0"></PyPI>',
      '<DockerHub url="https://hub.docker.com/_/nginx"></DockerHub>',
    ].join("\n");

    const html = await enrichProviderPackageEmbeds(input, {}, fetchImpl);
    expect(html).toContain('title="vite"');
    expect(html).toContain('version="7.0.0"');
    expect(html).toContain('license="MIT"');
    expect(html).toContain('repository="https://github.com/vitejs/vite"');
    expect(html).toContain('downloads="123456"');
    expect(html).toContain('stars="321"');
    expect(html).toContain('dateLabel="2026-08-26"');

    await enrichProviderPackageEmbeds(input, {}, fetchImpl);
    expect(requests).toEqual([
      "https://registry.npmjs.org/vite",
      "https://crates.io/api/v1/crates/serde",
      "https://pypi.org/pypi/requests/2.32.0/json",
      "https://hub.docker.com/v2/repositories/library/nginx",
    ]);
  });

  it("keeps explicit versions as separate cache entries", async () => {
    const requests: string[] = [];
    const fetchImpl: ProviderPackageFetch = async (input) => {
      const url = requestUrl(input);
      requests.push(url);
      return okJson({
        name: "vite",
        versions: {
          "6.0.0": { version: "6.0.0" },
          "7.0.0": { version: "7.0.0" },
        },
      });
    };

    await enrichProviderPackageEmbeds(
      '<NpmPackage url="https://www.npmjs.com/package/vite/v/6.0.0"></NpmPackage>',
      {},
      fetchImpl,
    );
    await enrichProviderPackageEmbeds(
      '<NpmPackage url="https://www.npmjs.com/package/vite/v/7.0.0"></NpmPackage>',
      {},
      fetchImpl,
    );
    await enrichProviderPackageEmbeds(
      '<NpmPackage url="https://www.npmjs.com/package/vite/v/6.0.0"></NpmPackage>',
      {},
      fetchImpl,
    );

    expect(requests).toEqual([
      "https://registry.npmjs.org/vite",
      "https://registry.npmjs.org/vite",
    ]);
  });

  it("keeps package tags literal with a diagnostic when metadata is unavailable", async () => {
    const warnings: string[] = [];
    console.warn = (message?: unknown) => {
      warnings.push(String(message));
    };
    const input = '<NpmPackage url="https://www.npmjs.com/package/private"></NpmPackage>';
    const html = await enrichProviderPackageEmbeds(input, {}, async () => {
      return new Response("{}", { status: 429 });
    });

    expect(html).toBe(input);
    expect(warnings[0]).toContain("429");
    expect(warnings[0]).toContain("link-only package card");
  });
});

function metadataFor(url: string): unknown {
  switch (url) {
    case "https://registry.npmjs.org/vite":
      return {
        name: "vite",
        description: "Next generation frontend tooling",
        "dist-tags": { latest: "7.0.0" },
        versions: { "7.0.0": { version: "7.0.0", license: "MIT" } },
        repository: { url: "git+https://github.com/vitejs/vite.git" },
        time: { "7.0.0": "2026-08-26T01:02:03.000Z" },
      };
    case "https://crates.io/api/v1/crates/serde":
      return {
        crate: {
          name: "serde",
          max_version: "1.0.228",
          description: "Serialization framework",
          license: "MIT OR Apache-2.0",
          repository: "https://github.com/serde-rs/serde",
          downloads: 123456,
          updated_at: "2026-08-25T00:00:00Z",
        },
      };
    case "https://pypi.org/pypi/requests/2.32.0/json":
      return {
        info: {
          name: "requests",
          version: "2.32.0",
          summary: "Python HTTP for Humans.",
          license: "Apache-2.0",
          project_urls: { Source: "https://github.com/psf/requests" },
        },
        urls: [{ upload_time_iso_8601: "2026-08-24T03:04:05.000000Z" }],
      };
    case "https://hub.docker.com/v2/repositories/library/nginx":
      return {
        namespace: "library",
        name: "nginx",
        description: "Official build of NGINX.",
        pull_count: 987654,
        star_count: 321,
        last_updated: "2026-08-23T10:11:12.000000Z",
      };
    default:
      throw new Error(`unexpected request ${url}`);
  }
}

function requestUrl(input: Parameters<typeof fetch>[0]): string {
  if (typeof input === "string") return input;
  if (input instanceof URL) return input.href;
  return input.url;
}

function okJson(value: unknown): Response {
  return new Response(JSON.stringify(value), {
    status: 200,
    headers: { "content-type": "application/json" },
  });
}
