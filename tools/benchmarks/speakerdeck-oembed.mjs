// Bundle src/plugins/speaker-deck.ts with Rolldown, then pass the bundle path.
// Uses local HTTP with a controlled delay; does not contact SpeakerDeck.
import { createServer } from "node:http";
import { performance } from "node:perf_hooks";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { createHash } from "node:crypto";

const { enrichSpeakerDeckEmbeds } = await import(pathToFileURL(resolve(process.argv[2])).href);
let requests = 0;
const server = createServer((_request, response) => {
  requests++;
  setTimeout(() => {
    response.setHeader("content-type", "application/json");
    response.end(
      JSON.stringify({
        title: "Local benchmark",
        author_name: "Example",
        html: '<iframe src="https://speakerdeck.com/player/0123456789abcdef"></iframe>',
      }),
    );
  }, 5);
});
await new Promise((done) => server.listen(0, "127.0.0.1", done));
try {
  const endpoint = `http://127.0.0.1:${server.address().port}/`;
  const fetcher = (url, init) => fetch(endpoint + new URL(url).search, init);
  for (const [name, count, unique] of [
    ["absent", 0, 0],
    ["repeated", 100, 1],
    ["mixed", 100, 10],
    ["distinct", 100, 100],
  ]) {
    const html =
      "<p>İ本文</p>" +
      Array.from(
        { length: count },
        (_, i) =>
          `<SpeakerDeck url="https://speakerdeck.com/docs/talk-${i % unique}"></SpeakerDeck>`,
      ).join("");
    const expected = await enrichSpeakerDeckEmbeds(html, fetcher);
    const samples = [];
    const counts = [];
    for (let i = 0; i < 7; i++) {
      requests = 0;
      const start = performance.now();
      const output = await enrichSpeakerDeckEmbeds(html, fetcher);
      samples.push(performance.now() - start);
      counts.push(requests);
      if (output !== expected) throw new Error("Output changed between samples");
    }
    console.log(
      JSON.stringify({
        name,
        bytes: Buffer.byteLength(html),
        samples_ms: samples,
        median_ms: [...samples].sort((a, b) => a - b)[3],
        requests: counts,
        sha256: createHash("sha256").update(expected).digest("hex"),
      }),
    );
  }
} finally {
  server.closeAllConnections();
  await new Promise((done) => server.close(done));
}
