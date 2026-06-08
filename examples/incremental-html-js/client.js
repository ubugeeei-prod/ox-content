import {
  createIncrementalMarkdownDomRenderer,
  injectIncrementalMarkdownDomStyles,
} from "/incremental-dom.js";

const source = document.querySelector("#source");
const preview = document.querySelector("#preview");
const committedBytes = document.querySelector("#committedBytes");
const pendingBytes = document.querySelector("#pendingBytes");
const status = document.querySelector("#status");
const replay = document.querySelector("#replay");
const stop = document.querySelector("#stop");

injectIncrementalMarkdownDomStyles();

let events;
const domRenderer = createIncrementalMarkdownDomRenderer({
  preview,
  source,
  animation: {
    mode: "character",
    durationMs: 170,
    staggerMs: 2,
  },
});

function formatBytes(value) {
  return `${value} B`;
}

function setRunning(running) {
  replay.disabled = running;
  stop.disabled = !running;
}

function resetView() {
  domRenderer.reset();
  committedBytes.textContent = "0 B";
  pendingBytes.textContent = "0 B";
  status.textContent = "idle";
}

function applyUpdate(payload, label) {
  domRenderer.apply(payload.result, { chunk: payload.chunk });
  committedBytes.textContent = formatBytes(payload.result.committedBytes);
  pendingBytes.textContent = formatBytes(payload.result.pendingBytes);
  status.textContent = label;
  source.scrollTop = source.scrollHeight;
}

function start() {
  if (events) {
    events.close();
  }
  resetView();
  setRunning(true);

  events = new EventSource("/stream");
  events.addEventListener("chunk", (event) => {
    applyUpdate(JSON.parse(event.data), "streaming");
  });
  events.addEventListener("finish", (event) => {
    applyUpdate(JSON.parse(event.data), "finished");
    events.close();
    events = undefined;
    setRunning(false);
  });
  events.addEventListener("error", () => {
    status.textContent = "connection closed";
    setRunning(false);
  });
}

replay.addEventListener("click", start);
stop.addEventListener("click", () => {
  if (events) {
    events.close();
    events = undefined;
  }
  status.textContent = "stopped";
  setRunning(false);
});

start();
