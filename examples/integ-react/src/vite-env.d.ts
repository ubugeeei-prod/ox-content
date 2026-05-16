/// <reference types="vite-plus/client" />

declare module "*.md" {
  import type { ComponentType } from "react";

  const Component: ComponentType<Record<string, unknown>>;
  export default Component;
}
