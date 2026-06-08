import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      embeds: {
        github: {
          token: process.env.GITHUB_TOKEN,
        },
        openGraph: {
          timeout: 5000,
        },
        pm: { sync: true },
        bluesky: true,
        twitter: true,
      },
    }),
  ],
});
