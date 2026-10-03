import { spawn } from "node:child_process";
import { access, rename, rm, writeFile } from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { frontmatterFixture } from "./oxct-frontmatter-fixture.mjs";
import { readMessages, writeMessage } from "../bin/oxct-lsp-protocol.mjs";

const bin = fileURLToPath(new URL("../bin/oxct.mjs", import.meta.url));
const cleanup = [];
afterEach(async () => {
  for (const task of cleanup.splice(0).reverse()) await task();
});

async function server(project = true) {
  const fixture = await frontmatterFixture();
  cleanup.push(() => rm(fixture.root, { recursive: true, force: true }));
  const child = spawn(process.execPath, [bin, "lsp", ...(project ? ["--project"] : [])], {
    cwd: fixture.root,
    stdio: ["pipe", "pipe", "pipe"],
  });
  cleanup.push(async () => {
    if (child.exitCode === null) {
      child.kill("SIGTERM");
      await new Promise((resolve) => child.once("exit", resolve));
    }
  });
  let stderr = "",
    error;
  child.stderr.on("data", (chunk) => {
    stderr += chunk;
  });
  child.on("error", (value) => {
    error = value;
  });
  const messages = [],
    listeners = new Set();
  readMessages(
    child.stdout,
    (message) => {
      messages.push(message);
      for (const listener of listeners) listener();
    },
    (value) => {
      error = value;
    },
  );
  const wait = (predicate) =>
    new Promise((resolve, reject) => {
      const poll = () => {
        const result = messages.find(predicate);
        if (!result && !error) return;
        clearTimeout(timer);
        listeners.delete(poll);
        if (error) reject(error);
        else resolve(result);
      };
      const timer = setTimeout(() => {
        listeners.delete(poll);
        reject(new Error(`LSP timeout: ${stderr}`));
      }, 15000);
      listeners.add(poll);
      poll();
    });
  const send = (method, params, id) =>
    writeMessage(child.stdin, {
      jsonrpc: "2.0",
      method,
      params,
      ...(id === undefined ? {} : { id }),
    });
  send("initialize", { rootUri: pathToFileURL(fixture.root).href, capabilities: {} }, 1);
  expect((await wait((message) => message.id === 1)).result.capabilities.hoverProvider).toBe(true);
  send("initialized", {});
  const open = (text, version = 1) =>
    send("textDocument/didOpen", {
      textDocument: { uri: fixture.uri, languageId: "markdown", version, text },
    });
  const change = (text, version) =>
    send("textDocument/didChange", {
      textDocument: { uri: fixture.uri, version },
      contentChanges: [{ text }],
    });
  const diagnostic = (version, predicate) =>
    wait(
      (message) =>
        message.method === "textDocument/publishDiagnostics" &&
        message.params.uri === fixture.uri &&
        message.params.version === version &&
        predicate(message.params.diagnostics),
    );
  return { ...fixture, child, send, wait, open, change, diagnostic, messages };
}

describe("Standard Schema project LSP", () => {
  it("merges async schema diagnostics, completion and native Markdown features", async () => {
    const client = await server();
    client.open("---\ntitle: bad\ncategory: \n---\n# Body\n");
    const report = await client.diagnostic(1, (items) =>
      items.some((item) => item.code === "frontmatter-standard-schema"),
    );
    expect(
      report.params.diagnostics.filter((item) => item.code.startsWith("frontmatter-")),
    ).toHaveLength(1);
    expect(report.params.diagnostics[0]).toMatchObject({
      range: { start: { line: 1, character: 7 } },
    });
    client.send(
      "textDocument/completion",
      { textDocument: { uri: client.uri }, position: { line: 2, character: 10 } },
      2,
    );
    expect(
      (await client.wait((message) => message.id === 2)).result.map((item) => item.label),
    ).toEqual(["guide", "news"]);
    client.send(
      "textDocument/hover",
      { textDocument: { uri: client.uri }, position: { line: 1, character: 3 } },
      3,
    );
    expect((await client.wait((message) => message.id === 3)).result.contents.value).toContain(
      "Accepted title",
    );
    client.send("textDocument/documentSymbol", { textDocument: { uri: client.uri } }, 4);
    expect(
      (await client.wait((message) => message.id === 4)).result.some(
        (item) => item.name === "Body",
      ),
    ).toBe(true);
    client.send("shutdown", null, 5);
    await client.wait((message) => message.id === 5);
    const exited = new Promise((resolve) => client.child.once("exit", (code) => resolve(code)));
    client.send("exit");
    expect(await exited).toBe(0);
  }, 25000);

  it("accepts non-cloneable transform output, rejects stale results and reloads configuration", async () => {
    const client = await server();
    client.open("---\ntitle: slow\n---\n", 1);
    client.change("---\ntitle: good\n---\n", 2);
    await client.diagnostic(
      2,
      (items) => !items.some((item) => item.code.startsWith("frontmatter-")),
    );
    await new Promise((resolve) => setTimeout(resolve, 250));
    expect(
      client.messages
        .filter((message) => message.params?.version === 2)
        .flatMap((message) => message.params.diagnostics ?? [])
        .some((item) => item.code === "frontmatter-standard-schema"),
    ).toBe(false);
    const updated = client.config.replace('["bad", "slow"]', '["bad", "slow", "good"]');
    await writeFile(`${client.configFile}.tmp`, updated);
    await rename(`${client.configFile}.tmp`, client.configFile);
    await client.diagnostic(2, (items) =>
      items.some((item) => item.code === "frontmatter-standard-schema"),
    );
    client.send("textDocument/didClose", { textDocument: { uri: client.uri } });
    await client.wait(
      (message) =>
        message.method === "textDocument/publishDiagnostics" &&
        message.params.uri === client.uri &&
        message.params.version === undefined &&
        !message.params.diagnostics.length,
    );
  }, 25000);

  it("does not execute Vite configuration without explicit project mode", async () => {
    const client = await server(false);
    await expect(access(client.marker)).rejects.toThrow();
  }, 25000);
});
