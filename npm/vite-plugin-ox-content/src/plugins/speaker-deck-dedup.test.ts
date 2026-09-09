import { describe, expect, it } from "vite-plus/test";
import { enrichSpeakerDeckEmbeds } from "./speaker-deck";

const URL = "https://speakerdeck.com/docs/talk";
const tag = (attrs = `url="${URL}"`) => `<SpeakerDeck ${attrs}>İ本文</SpeakerDeck>`;
const response = () =>
  new Response(
    JSON.stringify({
      title: "Fetched title",
      author_name: "Fetched author",
      html: '<iframe src="https://speakerdeck.com/player/0123456789abcdef"></iframe>',
    }),
  );

describe("SpeakerDeck document request sharing", () => {
  it("shares in-flight and completed requests while preserving each tag's attributes", async () => {
    let calls = 0;
    let resolve!: (value: Response) => void;
    const input =
      tag() + tag(`href="${URL}" title="Authored"`) + tag(`src="${URL}" author="Local"`).repeat(98);
    const pending = enrichSpeakerDeckEmbeds(input, () => {
      calls++;
      return new Promise<Response>((done) => {
        resolve = done;
      });
    });
    expect(calls).toBe(1);
    resolve(response());
    const html = await pending;
    expect(calls).toBe(1);
    expect(html.match(/player="0123456789abcdef"/g)).toHaveLength(100);
    expect(html.match(/İ本文/g)).toHaveLength(100);
    expect(html).toContain('title="Authored"');
    expect(html.match(/author="Local"/g)).toHaveLength(98);
    expect(html.indexOf('title="Fetched title"')).toBeLessThan(html.indexOf('title="Authored"'));
  });

  it("keeps distinct URL requests independent and bounded to four concurrent requests", async () => {
    let active = 0;
    let maximum = 0;
    const calls: string[] = [];
    const input = Array.from({ length: 12 }, (_, i) => tag(`url="${URL}-${i}"`)).join("");
    await enrichSpeakerDeckEmbeds(input, async (url) => {
      calls.push(url);
      maximum = Math.max(maximum, ++active);
      await new Promise<void>((resolve) => setTimeout(resolve, 0));
      active--;
      return response();
    });
    expect(new Set(calls).size).toBe(12);
    expect(maximum).toBe(4);
  });

  it("does not request metadata for player IDs or invalid URLs", async () => {
    const input =
      tag(`url="${URL}" player="01234567"`) +
      tag(`url="${URL}" id="01234567"`) +
      tag('url="https://example.com/docs/talk"');
    let calls = 0;
    expect(
      await enrichSpeakerDeckEmbeds(input, async () => {
        calls++;
        return response();
      }),
    ).toBe(input);
    expect(calls).toBe(0);
  });

  it("shares failure within one document but retries on the next document", async () => {
    let calls = 0;
    const input = tag().repeat(12);
    const fetcher = async () => {
      calls++;
      return new Response("{}", { status: 503 });
    };
    expect(await enrichSpeakerDeckEmbeds(input, fetcher)).toBe(input);
    expect(calls).toBe(1);
    expect(await enrichSpeakerDeckEmbeds(input, fetcher)).toBe(input);
    expect(calls).toBe(2);
  });

  it("does not retain successful metadata across documents", async () => {
    let calls = 0;
    const fetcher = async () => {
      calls++;
      return response();
    };
    await enrichSpeakerDeckEmbeds(tag().repeat(8), fetcher);
    await enrichSpeakerDeckEmbeds(tag().repeat(8), fetcher);
    expect(calls).toBe(2);
  });
});
