/// <reference types="svelte" />
/// <reference types="vite-plus/client" />

declare module "*.md" {
  const Component: import("svelte").ComponentType;
  export default Component;
}
