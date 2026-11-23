export const frameworkTestTasks = {
  "test:framework-integrations": {
    command: 'node -e ""',
    dependsOn: [
      "test:framework-hono",
      "test:framework-react",
      "test:framework-solid",
      "test:framework-svelte",
      "test:framework-vue",
    ],
  },
  "test:framework-hono": {
    command:
      "vp exec --filter @ox-content/vite-plugin-hono -- vp test src --exclude '**/node_modules/**'",
    dependsOn: ["build:framework-hono"],
  },
  "build:framework-hono": {
    command: "vp run --filter @ox-content/islands --filter @ox-content/vite-plugin-hono build",
    dependsOn: ["build:vite-plugin"],
  },
  "test:framework-react": {
    command: "vp exec --filter @ox-content/vite-plugin-react -- vp test src",
    dependsOn: ["build:vite-plugin"],
  },
  "test:framework-solid": {
    command: "vp exec --filter @ox-content/vite-plugin-solid -- vp test src",
    dependsOn: ["build:vite-plugin"],
  },
  "test:framework-svelte": {
    command: "vp exec --filter @ox-content/vite-plugin-svelte -- vp test src",
    dependsOn: ["build:vite-plugin"],
  },
  "test:framework-vue": {
    command: "vp exec --filter @ox-content/vite-plugin-vue -- vp test src",
    dependsOn: ["build:vite-plugin"],
  },
};
