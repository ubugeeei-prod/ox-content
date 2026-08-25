import { describe, expect, it } from "vite-plus/test";
import { canonicalizeFeedItemUrl, isBlockedFeedAddress, isSafeFeedUrl } from "./blog-feed-url";

describe("isSafeFeedUrl", () => {
  it("accepts public https URLs", () => {
    expect(isSafeFeedUrl("https://example.com/rss.xml")).toBe(true);
    expect(isSafeFeedUrl("  https://feeds.example.com/atom.xml  ")).toBe(true);
  });

  it("rejects non-https, credentials, and control characters", () => {
    expect(isSafeFeedUrl("http://example.com/rss.xml")).toBe(false);
    expect(isSafeFeedUrl("file:///tmp/feed.xml")).toBe(false);
    expect(isSafeFeedUrl("javascript:alert(1)")).toBe(false);
    expect(isSafeFeedUrl("ftp://example.com/rss.xml")).toBe(false);
    expect(isSafeFeedUrl("https://user:pass@example.com/rss.xml")).toBe(false);
    expect(isSafeFeedUrl("https://example.com/rss.xml\nnext")).toBe(false);
  });

  it("rejects private and loopback hosts before DNS", () => {
    expect(isSafeFeedUrl("https://localhost/rss.xml")).toBe(false);
    expect(isSafeFeedUrl("https://127.0.0.1/rss.xml")).toBe(false);
    expect(isSafeFeedUrl("https://10.0.0.4/rss.xml")).toBe(false);
    expect(isSafeFeedUrl("https://192.168.1.9/rss.xml")).toBe(false);
    expect(isSafeFeedUrl("https://172.16.1.4/rss.xml")).toBe(false);
    expect(isSafeFeedUrl("https://169.254.1.1/rss.xml")).toBe(false);
    expect(isSafeFeedUrl("https://[::1]/rss.xml")).toBe(false);
    expect(isSafeFeedUrl("https://feed.local/rss.xml")).toBe(false);
  });
});

describe("isBlockedFeedAddress", () => {
  it("rejects private and loopback resolved addresses", () => {
    expect(isBlockedFeedAddress("127.0.0.1")).toBe(true);
    expect(isBlockedFeedAddress("10.1.2.3")).toBe(true);
    expect(isBlockedFeedAddress("::1")).toBe(true);
    expect(isBlockedFeedAddress("::ffff:127.0.0.1")).toBe(true);
    expect(isBlockedFeedAddress("fd12::1")).toBe(true);
    expect(isBlockedFeedAddress("fe80::1")).toBe(true);
    expect(isBlockedFeedAddress("1.1.1.1")).toBe(false);
  });
});

describe("canonicalizeFeedItemUrl", () => {
  it("lowercases the host, drops hash and default port, and trims a trailing slash", () => {
    expect(canonicalizeFeedItemUrl("https://Example.com:443/Post/#frag")).toBe(
      "https://example.com/Post",
    );
    expect(canonicalizeFeedItemUrl("http://example.com/post")).toBeUndefined();
  });
});
