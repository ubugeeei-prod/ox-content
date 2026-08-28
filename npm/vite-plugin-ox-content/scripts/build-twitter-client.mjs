import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const distDir = join(packageRoot, "dist");
const runtimeSource = join(packageRoot, "../../crates/ox_content_ssg/src/html/plugins/twitter.js");

const runtime = await readFile(runtimeSource, "utf8");
const banner =
  "// Generated from crates/ox_content_ssg/src/html/plugins/twitter.js.\n" +
  "// Run npm/vite-plugin-ox-content/scripts/build-twitter-client.mjs.\n\n";
const declarations = `export type TweetCardsRoot = Document | Element;
export interface InitTweetCardsOptions {
  clipboard?: Pick<Clipboard, "writeText">;
  copiedMs?: number;
}
export declare const TWEET_COPY_RESET_MS = 6000;
export declare function initTweetCards(
  root?: TweetCardsRoot,
  options?: InitTweetCardsOptions,
): void;
export declare const initTwitterCards: typeof initTweetCards;
`;

await mkdir(distDir, { recursive: true });
await writeFile(
  join(distDir, "twitter-client.mjs"),
  `${banner}${runtime}\nexport { TWEET_COPY_RESET_MS, initTweetCards, initTwitterCards };\n`,
);
await writeFile(
  join(distDir, "twitter-client.cjs"),
  `${banner}"use strict";\n${runtime}\nmodule.exports = { TWEET_COPY_RESET_MS, initTweetCards, initTwitterCards };\n`,
);
await writeFile(join(distDir, "twitter-client.d.mts"), declarations);
await writeFile(join(distDir, "twitter-client.d.cts"), declarations);
