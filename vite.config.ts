import { spawnSync } from "node:child_process";
import { defineConfig } from "vite-plus";

const lifecycleEnvName = "OX_CONTENT_VP_LIFECYCLE";

const lifecycleTasks: Record<string, string> = {
  build: "build",
  check: "check",
  dev: "dev",
  fmt: "fmt",
};

// Vite+ direct commands bypass run.tasks, so root lifecycle commands delegate into the task graph.
const delegateLifecycleCommand = () => {
  if (process.env[lifecycleEnvName]) {
    return;
  }

  const [, , command, ...args] = process.argv;
  const taskName = command ? lifecycleTasks[command] : undefined;

  if (!taskName || args.length > 0) {
    return;
  }

  const result = spawnSync("vp", ["run", taskName], {
    env: { ...process.env, [lifecycleEnvName]: "1" },
    stdio: "inherit",
  });

  if (result.error) {
    throw result.error;
  }

  process.exit(result.status ?? 1);
};

delegateLifecycleCommand();

const task = (
  command: string,
  options: {
    cwd?: string;
    dependsOn?: string[];
    cache?: false;
  } = {},
) => ({
  command,
  ...options,
});

const noopTask = (
  dependsOn?: string[],
  options: {
    cache?: false;
  } = {},
) => task('node -e ""', { dependsOn, ...options });

const uncachedTask = (
  command: string,
  options: {
    cwd?: string;
    dependsOn?: string[];
  } = {},
) => task(command, { ...options, cache: false });

const vpBuiltin = (command: string) => `${lifecycleEnvName}=1 ${command}`;

export default defineConfig({
  fmt: {
    ignorePatterns: ["crates/ox_content_napi/index.d.ts", "crates/ox_content_ssg/templates/*.html"],
  },
  lint: {
    options: {
      typeAware: true,
    },
  },
  test: {
    passWithNoTests: true,
    exclude: ["**/test/vrt/**"],
  },
  run: {
    cache: {
      scripts: true,
      tasks: true,
    },
    tasks: {
      build: noopTask(["build:docs", "build:playground", "doc:cargo"]),
      "build:rust": task("cargo build --workspace"),
      "build:rust-release": task("cargo build --workspace --release"),
      "build:napi": task("vp run --filter ./crates/ox_content_napi build", {
        dependsOn: ["build:rust"],
      }),
      "build:npm": task("vp run --filter './npm/*' build", {
        dependsOn: ["build:napi"],
      }),
      "build:docs": task("vp run --filter ./docs build", {
        dependsOn: ["build:npm"],
      }),
      "build:playground": task("vp run --filter ./examples/playground build", {
        dependsOn: ["build:npm"],
      }),
      "build:wasm": task("node --experimental-strip-types scripts/build-wasm-package.ts"),

      test: noopTask(["test:rust", "test:ts"]),
      "test:rust": task("cargo test --workspace"),
      "test:rust-verbose": uncachedTask("cargo test --workspace -- --nocapture"),
      "test:ts": noopTask(["test:ts-unit", "test:vrt"]),
      "test:ts-unit": task("vp exec --filter @ox-content/vite-plugin -- vp test src", {
        dependsOn: ["build:napi"],
      }),
      "test:vrt": uncachedTask("vp exec --filter @ox-content/vite-plugin -- playwright test", {
        dependsOn: ["build:napi"],
      }),
      "test:vrt:update": uncachedTask(
        "vp exec --filter @ox-content/vite-plugin -- playwright test --update-snapshots",
        {
          dependsOn: ["build:napi"],
        },
      ),

      check: noopTask(["fmt:rust-check", "lint:rust", "check:ts"]),
      "check:rust": task("cargo check --workspace --all-targets"),

      clippy: task("cargo clippy --workspace --all-targets -- -D warnings", {
        dependsOn: ["check:rust"],
      }),

      fmt: noopTask(["fmt:rust", "fmt:ts"]),
      "fmt:rust": task("cargo fmt --all"),
      "fmt:check": noopTask(["fmt:rust-check", "fmt:ts-check"]),
      "fmt:rust-check": task("cargo fmt --all -- --check"),
      "fmt:ts": task(vpBuiltin("vp fmt")),
      "fmt:ts-check": task(vpBuiltin("vp fmt --check")),

      lint: noopTask(["lint:rust", "check:ts"]),
      "lint:rust": task("cargo clippy --workspace --all-targets -- -D warnings", {
        dependsOn: ["check:rust"],
      }),
      "lint:ts": noopTask(["check:ts"]),
      "check:ts": task(vpBuiltin("vp check")),

      bench: noopTask(["bench:rust", "bench:parse", "bench:bundle"], { cache: false }),
      "bench:rust": uncachedTask("cargo bench --workspace"),
      "bench:parse": uncachedTask("vp run --filter ./benchmarks/bundle-size benchmark:parse"),
      "bench:bundle": uncachedTask("vp run --filter ./benchmarks/bundle-size benchmark"),

      "doc:cargo": task("cargo doc --workspace --no-deps", {
        dependsOn: ["build:napi"],
      }),
      "doc:cargo-open": uncachedTask("cargo doc --workspace --no-deps --open"),
      clean: uncachedTask("cargo clean"),
      "napi-prepublish": uncachedTask("napi prepublish", {
        cwd: "crates/ox_content_napi",
      }),

      ci: noopTask(["fmt:rust-check", "lint:rust", "check:ts", "test"]),
      ready: noopTask(["fmt", "lint", "test"]),
      coverage: uncachedTask("cargo llvm-cov --workspace --html"),
      setup: uncachedTask(
        "rustup component add clippy rustfmt && cargo install cargo-watch cargo-llvm-cov",
      ),
      watch: uncachedTask("cargo watch -x test"),
      "watch-check": uncachedTask("cargo watch -x check"),

      playground: uncachedTask("vp run --filter ./examples/playground dev"),
      "integ-vue": uncachedTask("vp run --filter ./examples/integ-vue dev"),
      "integ-react": uncachedTask("vp run --filter ./examples/integ-react dev"),
      "integ-svelte": uncachedTask("vp run --filter ./examples/integ-svelte dev"),
      "ssg-vite": uncachedTask("vp run --filter ./examples/ssg-vite dev"),
      "plugin-markdown-it": uncachedTask("vp run --filter ./examples/plugin-markdown-it start"),
      "plugin-rehype": uncachedTask("vp run --filter ./examples/plugin-rehype start"),
      "gen-source-docs": uncachedTask("vp run --filter ./examples/gen-source-docs dev"),

      "dev:docs": uncachedTask("vp run --filter ./docs dev", {
        dependsOn: ["build:npm"],
      }),
      "dev:playground": uncachedTask("vp run --filter ./examples/playground dev"),
      dev: uncachedTask(
        [
          "trap 'kill 0' EXIT INT TERM",
          "vp run --filter ./docs dev &",
          "vp run --filter ./examples/playground dev &",
          "wait",
        ].join("\n"),
        {
          dependsOn: ["build:npm"],
        },
      ),
      "dev-build": noopTask(["build:docs"]),
      "dev-preview": uncachedTask("vp run --filter ./docs preview"),

      install: uncachedTask("vp install"),
      release: uncachedTask("node --experimental-strip-types scripts/release.ts"),
      "examples-install": noopTask(["install"], { cache: false }),
    },
  },
});
