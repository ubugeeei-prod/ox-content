import { spawn } from "node:child_process";
import { Worker } from "node:worker_threads";
import { fileURLToPath } from "node:url";
import { relative, resolve } from "node:path";
import { minimatch } from "minimatch";
import { applyChanges, readMessages, writeMessage } from "./oxct-lsp-protocol.mjs";
import { frontmatterCompletion, frontmatterHover } from "./oxct-frontmatter-completion.mjs";

/** Preserve the Rust server's Markdown features; execute project validators in an isolated worker. */
export async function runProjectLsp(args) {
  const options = { cwd: process.cwd(), config: undefined };
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--project") continue;
    if (args[i] === "--config" || args[i] === "-c") {
      options.config = args[++i];
      if (!options.config || options.config.startsWith("-"))
        throw new Error("Missing --config path");
    } else throw new Error(`Unknown lsp option: ${args[i]}`);
  }
  const child = spawn(
    process.execPath,
    [fileURLToPath(new URL("./oxct.mjs", import.meta.url)), "lsp"],
    { stdio: ["pipe", "pipe", "inherit"] },
  );
  const documents = new Map();
  const closed = new Set();
  let worker,
    project,
    epoch,
    configError,
    initializeResponse,
    initialized = false;
  let sequence = 0,
    stopped = false;
  const pending = new Map();
  const send = (message) => writeMessage(process.stdout, message);
  const forward = (message) => {
    if (!child.stdin.destroyed) writeMessage(child.stdin, message);
  };
  const report = (message) =>
    send({ jsonrpc: "2.0", method: "window/logMessage", params: { type: 1, message } });
  const fail = (error) => {
    console.error(`[ox-content] ${String(error)}`);
    stop(1);
  };

  function stop(code = 0) {
    if (stopped) return;
    stopped = true;
    child.kill();
    void worker?.terminate();
    process.exitCode = code;
    process.stdin.destroy();
  }
  child.on("error", fail);
  child.on("exit", (code, signal) => stop(code ?? (signal ? 1 : 0)));
  process.on("SIGTERM", () => stop());
  process.on("SIGINT", () => stop());
  process.stdin.on("end", () => {
    child.stdin.end();
    void worker?.terminate();
  });

  function selected(uri) {
    if (!project || !uri.startsWith("file:")) return undefined;
    const path = relative(project.root, fileURLToPath(uri)).replaceAll("\\", "/");
    if (path.startsWith("../") || path === "..") return undefined;
    return project.shapes.find(({ pattern }) => minimatch(path, pattern, { dot: true }));
  }

  function publish(uri, doc) {
    const match = doc.matched ?? !!selected(uri);
    const native = (doc.native ?? []).filter(
      (diagnostic) => !match || !String(diagnostic.code ?? "").startsWith("frontmatter-"),
    );
    const diagnostics =
      configError && match
        ? [
            {
              range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
              severity: 1,
              source: "ox-content",
              code: "frontmatter-config",
              message: configError,
            },
          ]
        : (doc.diagnostics ?? []);
    send({
      jsonrpc: "2.0",
      method: "textDocument/publishDiagnostics",
      params: {
        uri,
        version: doc.version,
        diagnostics: [...native, ...diagnostics],
      },
    });
  }

  function validate(uri, doc) {
    if (!worker || !uri.startsWith("file:")) return;
    const id = ++sequence;
    pending.delete(doc.request);
    doc.request = id;
    pending.set(id, { uri, version: doc.version });
    worker.postMessage({ type: "validate", id, source: doc.text, file: fileURLToPath(uri) });
  }

  function releaseInitialize() {
    if (!initializeResponse || !initialized) return;
    const capabilities = initializeResponse.result?.capabilities;
    if (capabilities?.completionProvider) {
      capabilities.completionProvider.triggerCharacters = [
        ...new Set([...(capabilities.completionProvider.triggerCharacters ?? []), ":", " "]),
      ];
    }
    send(initializeResponse);
    initializeResponse = undefined;
  }

  function startWorker(params) {
    const uri = params.workspaceFolders?.[0]?.uri ?? params.rootUri;
    if (uri?.startsWith("file:")) options.cwd = fileURLToPath(uri);
    else if (params.rootPath) options.cwd = resolve(params.rootPath);
    worker = new Worker(new URL("./oxct-frontmatter-worker.mjs", import.meta.url), {
      workerData: options,
      stdout: true,
      stderr: true,
    });
    worker.stdout.pipe(process.stderr);
    worker.stderr.pipe(process.stderr);
    worker.on("error", (error) => {
      configError = String(error);
      initialized = true;
      report(configError);
      releaseInitialize();
    });
    worker.on("message", (message) => {
      if (message.type === "ready" || message.type === "error") {
        epoch = message.epoch;
        initialized = true;
        configError = message.type === "error" ? message.message : undefined;
        if (message.type === "ready") project = message;
        else report(configError);
        releaseInitialize();
        for (const [uri, doc] of documents) {
          doc.diagnostics = [];
          if (message.type === "ready") doc.matched = !!selected(uri);
          validate(uri, doc);
          publish(uri, doc);
        }
      } else if (message.type === "result") {
        const request = pending.get(message.id);
        pending.delete(message.id);
        const doc = request && documents.get(request.uri);
        if (
          !doc ||
          doc.request !== message.id ||
          doc.version !== request.version ||
          epoch !== message.epoch
        )
          return;
        if (message.result) {
          doc.matched = message.result.matched;
          doc.diagnostics = message.result.diagnostics.map((issue) => ({
            range: {
              start: { line: issue.line - 1, character: issue.column - 1 },
              end: { line: issue.endLine - 1, character: issue.endColumn - 1 },
            },
            severity: 1,
            source: "ox-content",
            code: "frontmatter-standard-schema",
            message: issue.message,
          }));
        }
        publish(request.uri, doc);
      }
    });
  }

  readMessages(
    process.stdin,
    (message) => {
      const params = message.params ?? {};
      const uri = params.textDocument?.uri;
      if (message.method === "initialize") startWorker(params);
      if (message.method === "textDocument/didOpen") {
        closed.delete(uri);
        const doc = { text: params.textDocument.text, version: params.textDocument.version };
        documents.set(uri, doc);
        validate(uri, doc);
      } else if (message.method === "textDocument/didChange") {
        const doc = documents.get(uri);
        if (doc && params.textDocument.version > doc.version) {
          doc.text = applyChanges(doc.text, params.contentChanges);
          doc.version = params.textDocument.version;
          doc.diagnostics = [];
          validate(uri, doc);
        }
      } else if (message.method === "textDocument/didClose") {
        pending.delete(documents.get(uri)?.request);
        documents.delete(uri);
        closed.add(uri);
      } else if (
        message.method === "workspace/didChangeConfiguration" ||
        message.method === "workspace/didChangeWatchedFiles"
      )
        worker?.postMessage({ type: "reload" });
      if (
        message.id !== undefined &&
        ["textDocument/completion", "textDocument/hover"].includes(message.method)
      ) {
        const doc = documents.get(uri),
          schema = selected(uri);
        if (doc && schema) {
          const fn = message.method.endsWith("completion")
            ? frontmatterCompletion
            : frontmatterHover;
          const result = fn(schema.shape, doc.text, params.position);
          if (result !== undefined) {
            send({ jsonrpc: "2.0", id: message.id, result });
            return;
          }
        }
      }
      forward(message);
      if (message.method === "exit") child.stdin.end();
    },
    fail,
  );

  readMessages(
    child.stdout,
    (message) => {
      if (message.result?.capabilities && worker) {
        initializeResponse = message;
        releaseInitialize();
        return;
      }
      if (message.method === "textDocument/publishDiagnostics") {
        const { uri, version, diagnostics } = message.params;
        const doc = documents.get(uri);
        if (!doc) {
          if (!closed.has(uri) || !diagnostics.length) send(message);
          return;
        }
        if (version !== undefined && version !== doc.version) return;
        doc.native = diagnostics;
        publish(uri, doc);
        return;
      }
      send(message);
    },
    fail,
  );
}
