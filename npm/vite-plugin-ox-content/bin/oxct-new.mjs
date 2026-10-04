import { runNativeCli } from "./oxct-native.mjs";

export function runNew(args) {
  if (args.includes("--help") || args.includes("-h")) {
    console.log(`oxct new [directory]

Create a Vite/Ox Content site with a guided template, skin, palette, and package manager.

Options:
  --template docs|blog|minimal
  --skin <name>              Theme skin (default: editorial)
  --palette <name>           Color palette (default: nord)
  --package-manager npm|pnpm|yarn|bun|vp
  --install                 Install dependencies after creating the project
  --no-install              Only write the project
  --yes                     Use defaults without prompts
  -h, --help                Show this help`);
    return;
  }
  runNativeCli("new", args);
}
