use super::{all, vp};

#[test]
fn install_no_args() {
    let (npm, pnpm, yarn, bun) = all("npm install");
    assert_eq!(npm, "npm install");
    assert_eq!(pnpm, "pnpm install");
    assert_eq!(yarn, "yarn");
    assert_eq!(bun, "bun install");
}

#[test]
fn install_no_args_i_alias() {
    let (_, pnpm, yarn, bun) = all("npm i");
    assert_eq!(pnpm, "pnpm install");
    assert_eq!(yarn, "yarn");
    assert_eq!(bun, "bun install");
}

#[test]
fn install_package() {
    let (npm, pnpm, yarn, bun) = all("npm install vite");
    assert_eq!(npm, "npm install vite");
    assert_eq!(pnpm, "pnpm add vite");
    assert_eq!(yarn, "yarn add vite");
    assert_eq!(bun, "bun add vite");
}

#[test]
fn install_package_i_alias() {
    let (_, pnpm, yarn, bun) = all("npm i vite");
    assert_eq!(pnpm, "pnpm add vite");
    assert_eq!(yarn, "yarn add vite");
    assert_eq!(bun, "bun add vite");
}

#[test]
fn dev_dependency_short_flag() {
    let (_, pnpm, yarn, bun) = all("npm install -D vite");
    assert_eq!(pnpm, "pnpm add -D vite");
    assert_eq!(yarn, "yarn add -D vite");
    assert_eq!(bun, "bun add -D vite");
}

#[test]
fn dev_dependency_long_flag() {
    let (_, pnpm, yarn, bun) = all("npm install --save-dev vite");
    assert_eq!(pnpm, "pnpm add -D vite");
    assert_eq!(yarn, "yarn add -D vite");
    assert_eq!(bun, "bun add -D vite");
}

#[test]
fn global_install() {
    let (_, pnpm, yarn, bun) = all("npm install -g typescript");
    assert_eq!(pnpm, "pnpm add -g typescript");
    assert_eq!(yarn, "yarn global add typescript");
    assert_eq!(bun, "bun add -g typescript");
}

#[test]
fn global_install_long_flag() {
    let (_, pnpm, yarn, bun) = all("npm install --global typescript");
    assert_eq!(pnpm, "pnpm add -g typescript");
    assert_eq!(yarn, "yarn global add typescript");
    assert_eq!(bun, "bun add -g typescript");
}

#[test]
fn uninstall_package() {
    let (npm, pnpm, yarn, bun) = all("npm uninstall lodash");
    assert_eq!(npm, "npm uninstall lodash");
    assert_eq!(pnpm, "pnpm remove lodash");
    assert_eq!(yarn, "yarn remove lodash");
    assert_eq!(bun, "bun remove lodash");
}

#[test]
fn run_script() {
    let (npm, pnpm, yarn, bun) = all("npm run build");
    assert_eq!(npm, "npm run build");
    assert_eq!(pnpm, "pnpm run build");
    assert_eq!(yarn, "yarn build");
    assert_eq!(bun, "bun run build");
}

#[test]
fn npx_exec() {
    let (npm, pnpm, yarn, bun) = all("npx vite");
    assert_eq!(npm, "npx vite");
    assert_eq!(pnpm, "pnpm dlx vite");
    assert_eq!(yarn, "yarn dlx vite");
    assert_eq!(bun, "bunx vite");
}

#[test]
fn preserves_versions_and_scopes() {
    let (_, pnpm, yarn, bun) = all("npm install @scope/pkg@1.2.3 left-pad@^2");
    assert_eq!(pnpm, "pnpm add @scope/pkg@1.2.3 left-pad@^2");
    assert_eq!(yarn, "yarn add @scope/pkg@1.2.3 left-pad@^2");
    assert_eq!(bun, "bun add @scope/pkg@1.2.3 left-pad@^2");
}

#[test]
fn preserves_extra_flags() {
    let (_, pnpm, _, _) = all("npm install -D vite vitest --foo");
    assert_eq!(pnpm, "pnpm add -D vite vitest --foo");
}

#[test]
fn passthrough_lifecycle_scripts() {
    let (_, pnpm, yarn, bun) = all("npm test");
    assert_eq!(pnpm, "pnpm test");
    assert_eq!(yarn, "yarn test");
    assert_eq!(bun, "bun test");
}

#[test]
fn vp_conversions() {
    assert_eq!(vp("npm install"), "vp install");
    assert_eq!(vp("npm install vite"), "vp install vite");
    assert_eq!(vp("npm install -D vite"), "vp install -D vite");
    assert_eq!(vp("npm install -g typescript"), "vp install -g typescript");
    assert_eq!(vp("npm uninstall vite"), "vp uninstall vite");
    assert_eq!(vp("npm run build"), "vp run build");
    assert_eq!(vp("npx vite"), "vp exec -- vite");
    assert_eq!(vp("npm test"), "vp test");
}
