/**
 * Loader for CommonMark-style `spec.txt` fixtures.
 *
 * Mirrors `crates/ox_content_renderer/tests/spec_support/spec_txt.rs` so the JS
 * conformance runner and the Rust conformance suite read the exact same
 * examples out of the exact same vendored spec file.
 *
 * The spec stores each case as a fenced block:
 *
 *     ```````````````````````````````` example
 *     markdown input
 *     .
 *     expected html
 *     ````````````````````````````````
 *
 * Tabs are encoded as `→` so they survive editors that expand tabs; they are
 * converted back here.
 */

const FENCE = "`".repeat(32);

/**
 * @typedef {Object} SpecExample
 * @property {number} number 1-based example number, matching official numbering
 * @property {string} section Title of the closest enclosing ATX heading
 * @property {string} markdown Markdown input fed to the parser
 * @property {string} html Expected HTML output as printed in the spec
 */

/**
 * @param {string} text
 * @returns {SpecExample[]}
 */
export function parseSpec(text) {
  const examples = [];
  const lines = text.split("\n");
  let section = "";

  for (let index = 0; index < lines.length; index++) {
    const line = lines[index];

    if (line.startsWith("#")) {
      section = line.replace(/^#+/, "").trim();
      continue;
    }

    if (!line.startsWith(FENCE)) continue;
    const rest = line.slice(FENCE.length).trim();
    if (rest !== "example" && !rest.startsWith("example ")) continue;

    let markdown = "";
    let html = "";
    let inHtml = false;

    for (index++; index < lines.length; index++) {
      const bodyLine = lines[index];
      if (bodyLine.startsWith(FENCE)) break;
      if (!inHtml && bodyLine === ".") {
        inHtml = true;
        continue;
      }
      const decoded = bodyLine.replaceAll("→", "\t") + "\n";
      if (inHtml) html += decoded;
      else markdown += decoded;
    }

    examples.push({ number: examples.length + 1, section, markdown, html });
  }

  return examples;
}
