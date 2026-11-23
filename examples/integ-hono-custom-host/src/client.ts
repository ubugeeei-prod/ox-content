/// <reference types="@ox-content/vite-plugin-hono/virtual" />
import modules from "virtual:ox-content-hono/modules";
import { initHonoIslands } from "@ox-content/vite-plugin-hono/client";

const controller = initHonoIslands({ modules });
// Also dispose pending work when Vite replaces this entry during development.
if (import.meta.hot) import.meta.hot.dispose(() => controller.destroy());
