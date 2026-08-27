import { describe, expect, it } from "vite-plus/test";
import { importNapiModuleSync } from "../napi";
import { extractVideoId, transformYouTube } from "./youtube";

const ID = "dQw4w9WgXcQ";

const ACCEPTED: [string, string][] = [
  ["a bare id", ID],
  ["a watch URL", `https://www.youtube.com/watch?v=${ID}`],
  ["a watch URL with extra query", `https://www.youtube.com/watch?v=${ID}&t=30s`],
  ["a share URL", `https://youtu.be/${ID}`],
  ["an embed URL", `https://www.youtube.com/embed/${ID}`],
  ["a legacy /v/ URL", `https://www.youtube.com/v/${ID}`],
  ["a shorts URL", `https://www.youtube.com/shorts/${ID}`],
  ["a shorts URL with query", `https://www.youtube.com/shorts/${ID}?feature=share`],
];

const REJECTED = [
  "",
  "not a video",
  "https://vimeo.com/123456789",
  "https://www.youtube.com/watch?v=short",
  "https://www.youtube.com/results?search_query=x",
];

describe("extractVideoId", () => {
  for (const [label, input] of ACCEPTED) {
    it(`accepts ${label}`, () => {
      expect(extractVideoId(input)).toBe(ID);
    });
  }

  it("rejects inputs that name no video", () => {
    for (const input of REJECTED) {
      expect(extractVideoId(input), input).toBeNull();
    }
  });

  // An id is read as the first eleven id-shaped characters after a known
  // prefix, so an over-long `v=` value is truncated rather than refused. That
  // is what the regex implementation did and what the element rewrite still
  // does, so it is pinned here rather than left to be rediscovered.
  it("takes the first eleven characters of an over-long id", () => {
    expect(extractVideoId("https://www.youtube.com/watch?v=waytoolongtobeanid")).toBe(
      "waytoolongt",
    );
  });

  // The exported helper and the `<youtube>` rewrite must not drift apart: both
  // are the same Rust rule, and a page that renders an embed has to agree with
  // a caller that asked for the id directly.
  it("agrees with the native binding it delegates to", () => {
    const napi = importNapiModuleSync();
    for (const input of [...ACCEPTED.map(([, value]) => value), ...REJECTED]) {
      expect(extractVideoId(input), input).toBe(napi.extractYoutubeVideoId(input));
    }
  });

  it("agrees with the ids the element rewrite resolves", async () => {
    for (const [, input] of ACCEPTED) {
      const html = await transformYouTube(`<youtube url="${input}"></youtube>`);
      expect(html, input).toContain(`/embed/${extractVideoId(input)}`);
    }

    for (const input of REJECTED.filter((value) => value !== "")) {
      const source = `<youtube url="${input}"></youtube>`;
      expect(await transformYouTube(source), input).toBe(source);
    }
  });
});
