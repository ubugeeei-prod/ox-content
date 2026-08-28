import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { enhanceTweetCopyActions, initTweetCards } from "./copy";
import { formatFullDate } from "./date-utils";
import { clearTweetCache } from "./fetch";
import { transformFetchedTweets } from "./transform";
import type { TwitterEmbedOptions } from "./types";

const FULL_MEDIA_CSS = path.join(
  import.meta.dirname,
  "../../../../../crates/ox_content_ssg/src/plugins/social-tweet-full-media.css",
);
const FIXTURE_ID = "1941072675872641440";
const FIXTURE_CREATED_AT = "2025-07-04T09:52:43.000Z";
const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
  clearTweetCache();
});

describe("full-card react-tweet parity", () => {
  it("keeps replies as a full-width react-tweet control", async () => {
    const html = await renderCard(
      {
        text: "Hello",
        id_str: FIXTURE_ID,
        conversation_count: 12,
        user: user(),
      },
      { appearance: "full" },
      { html: `<XPost id="${FIXTURE_ID}" />` },
    );
    expect(html).toContain('class="ox-tweet__replies"');
    expect(html).toContain('class="ox-tweet__replies-link"');
    expect(html).toContain("Read 12 replies");

    const css = await readFile(FULL_MEDIA_CSS, "utf8");
    expect(css).toMatch(
      /\.ox-tweet--full \.ox-tweet__replies-link \{\n  box-sizing: border-box;\n  display: flex;/,
    );
    expect(css).toContain("\n  width: 100%;\n  min-height: 32px;");
    expect(css).not.toMatch(/\.ox-tweet__replies-link \{[^}]*display:\s*inline-flex/);
    expect(css).not.toMatch(/\.ox-tweet--full \.ox-tweet__replies \{[^}]*text-align:\s*center/);
    expect(css).toContain(".ox-tweet--full .ox-tweet__replies-link:hover");
  });

  it("uses the sveltweet copied checkmark state for Copy link", async () => {
    const [baseCss, mediaCss, html] = await Promise.all([
      readFile(
        path.join(
          import.meta.dirname,
          "../../../../../crates/ox_content_ssg/src/plugins/social-tweet-full.css",
        ),
        "utf8",
      ),
      readFile(FULL_MEDIA_CSS, "utf8"),
      renderCard(
        {
          text: "Hello",
          id_str: FIXTURE_ID,
          conversation_count: 12,
          user: user(),
        },
        { appearance: "full" },
        { html: `<XPost id="${FIXTURE_ID}" />` },
      ),
    ]);

    expect(html).toContain('data-ox-tweet-copy-status role="status" aria-live="polite"');
    expect(baseCss).toContain("--ox-tweet-icon-check");
    expect(baseCss).toContain(
      ".ox-tweet--full .ox-tweet__action--copy[data-ox-tweet-copied] .ox-tweet__icon--copy",
    );
    expect(mediaCss).toContain(".ox-tweet--full .ox-tweet__copy-status");
    expect(mediaCss).toContain(
      ".ox-tweet--full .ox-tweet__action--copy[data-ox-tweet-copied] .ox-tweet__copied-text",
    );
  });

  it("formats full-card timestamps in UTC by default and honors timeZone", async () => {
    expect(formatFullDate(FIXTURE_CREATED_AT)).toEqual({
      iso: FIXTURE_CREATED_AT,
      label: "9:52 AM · Jul 4, 2025",
    });
    expect(formatFullDate(FIXTURE_CREATED_AT, "Europe/London")?.label).toBe(
      "10:52 AM · Jul 4, 2025",
    );
    expect(formatFullDate(FIXTURE_CREATED_AT, "Not/AZone")?.label).toBe("9:52 AM · Jul 4, 2025");

    const post = {
      text: "Hello from London",
      id_str: FIXTURE_ID,
      created_at: FIXTURE_CREATED_AT,
      user: user(),
    };
    const utc = await renderCard(
      post,
      { appearance: "full" },
      { html: `<XPost id="${FIXTURE_ID}" />` },
    );
    expect(utc).toContain("9:52 AM · Jul 4, 2025");
    expect(utc).toContain(`datetime="${FIXTURE_CREATED_AT}"`);

    const london = await renderCard(
      post,
      { appearance: "full", timeZone: "Europe/London" },
      { html: `<XPost id="${FIXTURE_ID}" />` },
    );
    expect(london).toContain("10:52 AM · Jul 4, 2025");
  });

  it("enhances Copy link to Copied! with delegated, repeat-safe listeners", async () => {
    const written: string[] = [];
    const action = fakeCopyAction("https://x.com/i/web/status/1941072675872641440");
    const root = fakeCopyRoot();
    const stop = enhanceTweetCopyActions(root, {
      clipboard: {
        writeText: async (value: string) => {
          written.push(value);
        },
      },
      copiedMs: 5,
    });
    initTweetCards(root, { clipboard: { writeText: async (value) => written.push(value) } });

    const event = await root.dispatch(action);
    expect(event.defaultPrevented).toBe(true);
    expect(written).toEqual(["https://x.com/i/web/status/1941072675872641440"]);
    expect(action.getAttribute("data-ox-tweet-copied")).toBe("");
    expect(action.getAttribute("aria-label")).toBe("Copied!");
    expect(action.status.textContent).toBe("Copied!");

    await new Promise((resolve) => setTimeout(resolve, 10));
    expect(action.getAttribute("data-ox-tweet-copied")).toBeNull();
    expect(action.getAttribute("aria-label")).toBe("Copy link");
    expect(action.status.textContent).toBe("");
    stop();
  });

  it("keeps permalink fallback when Clipboard API is unavailable", async () => {
    const root = fakeCopyRoot();
    const action = fakeCopyAction("https://x.com/i/web/status/1941072675872641440");
    enhanceTweetCopyActions(root, { clipboard: undefined, copiedMs: 5 });

    const event = await root.dispatch(action);

    expect(event.defaultPrevented).toBe(false);
    expect(action.getAttribute("data-ox-tweet-copied")).toBeNull();
    expect(action.getAttribute("aria-label")).toBe("Copy link");
  });

  it("announces clipboard rejection without showing copied state", async () => {
    const root = fakeCopyRoot();
    const action = fakeCopyAction("https://x.com/i/web/status/1941072675872641440");
    enhanceTweetCopyActions(root, {
      clipboard: {
        writeText: async () => {
          throw new Error("denied");
        },
      },
      copiedMs: 5,
    });

    const event = await root.dispatch(action);

    expect(event.defaultPrevented).toBe(true);
    expect(action.getAttribute("data-ox-tweet-copied")).toBeNull();
    expect(action.getAttribute("aria-label")).toBe("Copy failed");
    expect(action.status.textContent).toBe("Copy failed");
  });

  it("delegates to dynamically inserted cards", async () => {
    const written: string[] = [];
    const root = fakeCopyRoot();
    enhanceTweetCopyActions(root, {
      clipboard: { writeText: async (value) => written.push(value) },
      copiedMs: 5,
    });

    const action = fakeCopyAction("https://x.com/i/web/status/1941072675872641440");
    await root.dispatch(action);

    expect(written).toEqual(["https://x.com/i/web/status/1941072675872641440"]);
  });
});

function user() {
  return { name: "Ox Content", screen_name: "ox_content" };
}

function fakeCopyRoot() {
  const listeners = new Map<string, Array<(event: FakeClickEvent) => unknown>>();
  return {
    addEventListener(type: string, listener: (event: FakeClickEvent) => unknown) {
      listeners.set(type, [...(listeners.get(type) ?? []), listener]);
    },
    removeEventListener(type: string, listener: (event: FakeClickEvent) => unknown) {
      listeners.set(
        type,
        (listeners.get(type) ?? []).filter((candidate) => candidate !== listener),
      );
    },
    async dispatch(target: FakeCopyAction): Promise<FakeClickEvent> {
      const event: FakeClickEvent = {
        defaultPrevented: false,
        target,
        preventDefault() {
          this.defaultPrevented = true;
        },
      };
      await Promise.all((listeners.get("click") ?? []).map((listener) => listener(event)));
      return event;
    },
  };
}

function fakeCopyAction(url: string): FakeCopyAction {
  const attributes = new Map<string, string>([
    ["data-ox-tweet-copy", ""],
    ["data-ox-tweet-copy-url", url],
    ["href", url],
    ["aria-label", "Copy link"],
  ]);
  const status = { textContent: "" };
  return {
    status,
    getAttribute(name: string) {
      return attributes.has(name) ? (attributes.get(name) ?? "") : null;
    },
    setAttribute(name: string, value: string) {
      attributes.set(name, value);
    },
    removeAttribute(name: string) {
      attributes.delete(name);
    },
    querySelector(selector: string) {
      return selector === "[data-ox-tweet-copy-status]" ? status : null;
    },
    closest(selector: string) {
      return selector === "[data-ox-tweet-copy]" ? this : null;
    },
  };
}

interface FakeClickEvent {
  defaultPrevented: boolean;
  target: FakeCopyAction;
  preventDefault(): void;
}

interface FakeCopyAction {
  status: { textContent: string };
  getAttribute(name: string): string | null;
  setAttribute(name: string, value: string): void;
  removeAttribute(name: string): void;
  querySelector(selector: string): { textContent: string } | null;
  closest(selector: string): FakeCopyAction | null;
}

async function renderCard(
  data: unknown,
  twitter: TwitterEmbedOptions = {},
  options?: { html?: string },
): Promise<string> {
  const root = await mkdtemp(path.join(tmpdir(), "ox-tweet-parity-"));
  globalThis.fetch = async (input) => {
    const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
    if (url.startsWith("https://cdn.syndication.twimg.com/")) {
      return { ok: true, json: async () => data } as Response;
    }
    return { ok: true, arrayBuffer: async () => new Uint8Array([1, 2, 3]).buffer } as Response;
  };
  try {
    return await transformFetchedTweets(options?.html ?? `<XPost id="${FIXTURE_ID}" />`, {
      fetch: true,
      cache: false,
      mediaOutputDir: path.join(root, "media"),
      mediaPublicPath: "/tweets",
      ...twitter,
    });
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}
