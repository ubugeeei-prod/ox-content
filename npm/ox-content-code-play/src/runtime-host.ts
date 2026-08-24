/** True when the current isolate can load `node:vm`. */
export function hasNodeVm(): boolean {
  return typeof process !== "undefined" && Boolean(process.versions?.node);
}
