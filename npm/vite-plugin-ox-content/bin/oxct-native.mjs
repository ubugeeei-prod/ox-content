import { fileURLToPath } from "node:url";
import { loadNapi } from "./oxct-napi.mjs";

// The same Rust implementation is distributed through the existing native packages.
export function runNativeCli(command, args) {
  const assets = fileURLToPath(new URL("../dist/editors/neovim", import.meta.url));
  const native = loadNapi();
  // Let Rust handle termination while its terminal loop blocks the JS event loop.
  // Node's default signal handler would exit before Rust can restore the terminal.
  const signals = command === "tui" && process.platform !== "win32" ? ["SIGINT", "SIGTERM"] : [];
  const deferSignal = () => {};
  for (const signal of signals) process.on(signal, deferSignal);
  try {
    process.exitCode = native.runAuthoringCli([command, ...args], assets);
  } finally {
    for (const signal of signals) process.off(signal, deferSignal);
  }
}
