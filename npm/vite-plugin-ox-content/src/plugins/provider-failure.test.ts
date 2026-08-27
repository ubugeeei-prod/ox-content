import { afterEach, beforeEach, describe, expect, it, vi } from "vite-plus/test";
import {
  classifyError,
  classifyStatus,
  describeFailure,
  resetEmbedFailureReporting,
  warnEmbedFailure,
} from "./provider-failure";

describe("classifying a fetch outcome", () => {
  it("separates a missing resource from a refusal", () => {
    expect(classifyStatus(404)).toEqual({ kind: "missing", status: 404 });
    expect(classifyStatus(410)).toEqual({ kind: "missing", status: 410 });
    expect(classifyStatus(403)).toEqual({ kind: "unavailable", status: 403 });
    expect(classifyStatus(401)).toEqual({ kind: "unavailable", status: 401 });
    expect(classifyStatus(429)).toEqual({ kind: "unavailable", status: 429 });
  });

  it("treats a server error as a failed request, not a refusal", () => {
    // 5xx says the provider is broken, not that it said no.
    expect(classifyStatus(500)).toEqual({ kind: "fetch-failed", reason: "HTTP 500" });
    expect(classifyStatus(502)).toEqual({ kind: "fetch-failed", reason: "HTTP 502" });
  });

  it("reads a missing network off the syscall error", () => {
    const offline = Object.assign(new Error("fetch failed"), {
      cause: { code: "ENOTFOUND" },
    });
    expect(classifyError(offline)).toEqual({ kind: "offline", reason: "ENOTFOUND" });
  });

  it("names a timeout as such", () => {
    const aborted = Object.assign(new Error("aborted"), { name: "AbortError" });
    expect(classifyError(aborted)).toEqual({ kind: "fetch-failed", reason: "timed out" });
  });

  it("does not mistake an ordinary error for being offline", () => {
    expect(classifyError(new Error("bad json"))).toEqual({
      kind: "fetch-failed",
      reason: "bad json",
    });
    expect(classifyError("not an error")).toEqual({
      kind: "fetch-failed",
      reason: "unknown error",
    });
  });
});

describe("what each cause reads as", () => {
  it("says something different for each", () => {
    expect(describeFailure(classifyStatus(404))).toContain("does not exist");
    expect(describeFailure(classifyStatus(403))).toContain("refused");
    expect(describeFailure(classifyStatus(429))).toContain("rate limiting");
    expect(describeFailure(classifyStatus(503))).toBe("HTTP 503");
    expect(describeFailure({ kind: "offline", reason: "EAI_AGAIN" })).toContain("no network");
  });
});

describe("reporting", () => {
  let warned: string[];

  beforeEach(() => {
    resetEmbedFailureReporting();
    warned = [];
    vi.spyOn(console, "warn").mockImplementation((message: unknown) => {
      warned.push(String(message));
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("names the provider and the target for a per-embed failure", () => {
    warnEmbedFailure("npm", "left-pad", classifyStatus(404), "a link-only package card");
    expect(warned).toHaveLength(1);
    expect(warned[0]).toContain("npm");
    expect(warned[0]).toContain("left-pad");
    expect(warned[0]).toContain("does not exist");
    expect(warned[0]).toContain("a link-only package card");
  });

  it("reports an offline build once, not once per embed", () => {
    const offline = { kind: "offline", reason: "ENOTFOUND" } as const;
    for (let index = 0; index < 20; index++) {
      warnEmbedFailure("npm", `package-${index}`, offline, "a link-only package card");
    }
    // The single fact that matters would otherwise be buried under twenty
    // identical lines.
    expect(warned).toHaveLength(1);
    expect(warned[0]).toContain("No network");
    expect(warned[0]).toContain("every embed lookup this build");
  });

  it("still reports other causes while offline has already been said", () => {
    warnEmbedFailure("npm", "a", { kind: "offline", reason: "ENOTFOUND" }, "a card");
    warnEmbedFailure("npm", "b", classifyStatus(404), "a card");
    expect(warned).toHaveLength(2);
    expect(warned[1]).toContain("does not exist");
  });

  it("does not name an offline provider or target, because neither is at fault", () => {
    warnEmbedFailure("npm", "left-pad", { kind: "offline", reason: "ENOTFOUND" }, "a card");
    expect(warned[0]).not.toContain("left-pad");
  });
});
