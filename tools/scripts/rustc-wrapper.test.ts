import { execFileSync } from "node:child_process";
import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";

const repoRoot = resolve(".");
const wrapperPath = "tools/scripts/rustc-wrapper.sh";
const roots: string[] = [];
afterEach(() => roots.splice(0).forEach((root) => rmSync(root, { recursive: true, force: true })));

// A copy of the wrapper inside a scratch workspace, a fake compiler that
// reports what it received, and a fake kache that forwards to it.
function scratchWorkspace() {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "ox-content-rustc-wrapper-")));
  roots.push(root);
  const wrapper = join(root, wrapperPath);
  mkdirSync(dirname(wrapper), { recursive: true });
  copyFileSync(join(repoRoot, wrapperPath), wrapper);
  chmodSync(wrapper, 0o755);
  const rustc = join(root, "fake-rustc");
  writeFileSync(
    rustc,
    '#!/bin/sh\nprintf "bootstrap=%s\\n" "${RUSTC_BOOTSTRAP:-}"\nprintf "%s\\n" "$@"\n',
  );
  chmodSync(rustc, 0o755);
  const kache = join(root, "fake-kache");
  writeFileSync(kache, '#!/bin/sh\necho via-kache\nexec "$@"\n');
  chmodSync(kache, 0o755);
  return { root, wrapper, rustc, kache };
}

type Workspace = ReturnType<typeof scratchWorkspace>;

function run(workspace: Workspace, args: string[], env: Record<string, string>): string[] {
  return execFileSync(workspace.wrapper, [workspace.rustc, ...args], {
    env: { PATH: process.env.PATH, ...env },
    encoding: "utf8",
  })
    .trimEnd()
    .split("\n");
}

// Each case spawns a few shells; leave room for a loaded runner.
describe("rustc wrapper", { timeout: 30_000 }, () => {
  const crate = ["--crate-name", "ox_content_parser", "--edition=2024", "src/lib.rs"];
  const frontend = { OX_CONTENT_RUSTC_FRONTEND_JOBS: "8" };
  const memberEnv = (workspace: Workspace) => ({
    CARGO_MANIFEST_DIR: join(workspace.root, "crates/ox_content_parser"),
    CARGO_CRATE_NAME: "ox_content_parser",
  });

  it("adds the parallel frontend to a workspace crate", () => {
    const workspace = scratchWorkspace();
    expect(run(workspace, crate, { ...frontend, ...memberEnv(workspace) })).toEqual([
      "bootstrap=ox_content_parser",
      ...crate,
      "-Zunstable-options",
      "--jobs-frontend=8",
    ]);
  });

  it("compiles a registry dependency exactly as Cargo asked", () => {
    const workspace = scratchWorkspace();
    const dependency = ["--crate-name", "proc_macro2", "src/lib.rs"];
    expect(
      run(workspace, dependency, {
        ...frontend,
        CARGO_MANIFEST_DIR: "/home/runner/.cargo/registry/src/proc-macro2-1.0.107",
        CARGO_CRATE_NAME: "proc_macro2",
      }),
    ).toEqual(["bootstrap=", ...dependency]);
  });

  it("does not treat a sibling checkout whose path extends the workspace as a member", () => {
    const workspace = scratchWorkspace();
    expect(
      run(workspace, crate, {
        ...frontend,
        CARGO_MANIFEST_DIR: `${workspace.root}-base/crates/ox_content_parser`,
        CARGO_CRATE_NAME: "ox_content_parser",
      }),
    ).toEqual(["bootstrap=", ...crate]);
  });

  it("keeps unstable features away from build-script probes", () => {
    // A probe inherits the member's environment but names another crate.
    const workspace = scratchWorkspace();
    const probe = ["--crate-name", "probe", "--crate-type=lib", "-"];
    expect(
      run(workspace, probe, {
        ...frontend,
        ...memberEnv(workspace),
        CARGO_CRATE_NAME: "build_script_build",
      }),
    ).toEqual(["bootstrap=", ...probe]);
  });

  it("passes Cargo's version queries through", () => {
    const workspace = scratchWorkspace();
    expect(run(workspace, ["-vV"], { ...frontend, ...memberEnv(workspace) })).toEqual([
      "bootstrap=",
      "-vV",
    ]);
  });

  it("is a passthrough when the frontend is off", () => {
    const workspace = scratchWorkspace();
    for (const jobs of ["1", "0", ""]) {
      expect(
        run(workspace, crate, { OX_CONTENT_RUSTC_FRONTEND_JOBS: jobs, ...memberEnv(workspace) }),
      ).toEqual(["bootstrap=", ...crate]);
    }
  });

  it("hands every call to kache when the shell provides it", () => {
    const workspace = scratchWorkspace();
    expect(
      run(workspace, crate, {
        OX_CONTENT_KACHE: workspace.kache,
        OX_CONTENT_RUSTC_FRONTEND_JOBS: "4",
        ...memberEnv(workspace),
      }),
    ).toEqual([
      "via-kache",
      "bootstrap=ox_content_parser",
      ...crate,
      "-Zunstable-options",
      "--jobs-frontend=4",
    ]);
    expect(run(workspace, ["-vV"], { OX_CONTENT_KACHE: workspace.kache })).toEqual([
      "via-kache",
      "bootstrap=",
      "-vV",
    ]);
    expect(run(workspace, ["-vV"], { OX_CONTENT_KACHE: "" })).toEqual(["bootstrap=", "-vV"]);
  });
});

describe("rustc wrapper wiring", () => {
  const read = (file: string) => readFileSync(join(repoRoot, file), "utf8");

  it("routes the dev shell through the wrapper and kache", () => {
    const shell = read("tools/nix/dev-shell.nix");
    expect(shell).toContain(
      'export RUSTC_WRAPPER="$OX_CONTENT_WORKSPACE_ROOT/tools/scripts/rustc-wrapper.sh"',
    );
    expect(shell).toContain("config.packages.kache");
    // The parallel frontend runs on the rust-toolchain.toml compiler.
    expect(shell).not.toMatch(/rust-bin\.nightly/u);
  });

  it("opts the Rust CI jobs in, and keeps release builds out", () => {
    const action = read(".github/actions/rust-parallel-frontend/action.yml");
    expect(action).toContain("runner.os != 'Windows' && inputs.jobs != '1'");
    expect(action).toContain('wrapper="$GITHUB_WORKSPACE/tools/scripts/rustc-wrapper.sh"');
    expect(action).toContain('-z "${RUSTC_WRAPPER:-}" && -x "$wrapper"');

    const ci = read(".github/workflows/ci.yml");
    expect(ci.match(/uses: \.\/\.github\/actions\/rust-parallel-frontend/gu)).toHaveLength(2);
    for (const workflow of [
      "build-release-cli.yml",
      "build-release-napi.yml",
      "build-release-editors.yml",
      "benchmark.yml",
      "native-cli.yml",
    ]) {
      expect(read(`.github/workflows/${workflow}`), workflow).not.toContain(
        "rust-parallel-frontend",
      );
    }
  });
});
