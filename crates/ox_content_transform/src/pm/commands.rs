/// Convert an npm-style command to the equivalent for `target` (one of
/// `npm`/`pnpm`/`yarn`/`bun`/`vp`). `npm` returns the command unchanged.
/// Unknown shapes fall back to a best-effort binary swap so nothing silently
/// breaks.
pub(super) fn convert_command(command: &str, target: &str) -> String {
    if target == "npm" {
        return command.to_string();
    }

    let tokens: Vec<&str> = command.split_whitespace().collect();
    if tokens.is_empty() {
        return command.to_string();
    }

    // `npx <bin> ...`
    if tokens[0] == "npx" {
        let rest = &tokens[1..];
        return match target {
            "pnpm" => join("pnpm dlx", rest),
            "yarn" => join("yarn dlx", rest),
            "bun" => join("bunx", rest),
            "vp" => join("vp exec --", rest),
            _ => command.to_string(),
        };
    }

    if tokens[0] != "npm" {
        return command.to_string();
    }
    if tokens.len() < 2 {
        return target.to_string();
    }

    let verb = tokens[1];
    let args = &tokens[2..];

    match verb {
        "install" | "i" | "add" => convert_add(args, target),
        "uninstall" | "remove" | "rm" | "un" => convert_remove(args, target),
        "run" => convert_run(args, target),
        _ => convert_passthrough(verb, args, target),
    }
}

/// Split args into (flags-affecting-verb, packages). We only special-case the
/// dependency-scope flags; every other flag is preserved verbatim with the
/// packages.
struct Scope {
    dev: bool,
    global: bool,
    /// All args except the scope flags we rewrite, in original order.
    rest: Vec<String>,
}

fn classify_scope(args: &[&str]) -> Scope {
    let mut dev = false;
    let mut global = false;
    let mut rest = Vec::with_capacity(args.len());
    for &arg in args {
        match arg {
            "-D" | "--save-dev" => dev = true,
            "-g" | "--global" => global = true,
            _ => rest.push(arg.to_string()),
        }
    }
    Scope { dev, global, rest }
}

fn convert_add(args: &[&str], target: &str) -> String {
    let scope = classify_scope(args);
    let has_packages = scope.rest.iter().any(|arg| !arg.starts_with('-'));

    // No packages means install/sync the project, e.g. `npm install`.
    if !has_packages && !scope.global {
        return match target {
            "pnpm" => with_rest("pnpm install", &scope.rest),
            "yarn" => with_rest("yarn", &scope.rest),
            "bun" => with_rest("bun install", &scope.rest),
            "vp" => with_rest("vp install", &scope.rest),
            _ => with_rest("npm install", &scope.rest),
        };
    }

    match target {
        "pnpm" => {
            let mut base = String::from("pnpm add");
            if scope.dev {
                base.push_str(" -D");
            }
            if scope.global {
                base.push_str(" -g");
            }
            with_rest(&base, &scope.rest)
        }
        "yarn" => {
            let mut base = String::from("yarn");
            if scope.global {
                base.push_str(" global add");
            } else {
                base.push_str(" add");
            }
            if scope.dev {
                base.push_str(" -D");
            }
            with_rest(&base, &scope.rest)
        }
        "bun" => {
            let mut base = String::from("bun add");
            if scope.dev {
                base.push_str(" -D");
            }
            if scope.global {
                base.push_str(" -g");
            }
            with_rest(&base, &scope.rest)
        }
        "vp" => {
            let mut base = String::from("vp install");
            if scope.dev {
                base.push_str(" -D");
            }
            if scope.global {
                base.push_str(" -g");
            }
            with_rest(&base, &scope.rest)
        }
        _ => with_rest("npm install", args),
    }
}

fn convert_remove(args: &[&str], target: &str) -> String {
    let scope = classify_scope(args);
    match target {
        "pnpm" => {
            with_rest(if scope.global { "pnpm remove -g" } else { "pnpm remove" }, &scope.rest)
        }
        "yarn" => {
            with_rest(if scope.global { "yarn global remove" } else { "yarn remove" }, &scope.rest)
        }
        "bun" => with_rest(if scope.global { "bun remove -g" } else { "bun remove" }, &scope.rest),
        "vp" => {
            with_rest(if scope.global { "vp uninstall -g" } else { "vp uninstall" }, &scope.rest)
        }
        _ => with_rest("npm uninstall", args),
    }
}

fn convert_run(args: &[&str], target: &str) -> String {
    match target {
        "pnpm" => with_rest("pnpm run", args),
        "yarn" => with_rest("yarn", args),
        "bun" => with_rest("bun run", args),
        "vp" => with_rest("vp run", args),
        _ => with_rest("npm run", args),
    }
}

fn convert_passthrough(verb: &str, args: &[&str], target: &str) -> String {
    let mut base = String::from(target);
    base.push(' ');
    base.push_str(verb);
    with_rest(&base, args)
}

fn with_rest(base: &str, rest: &[impl AsRef<str>]) -> String {
    let mut out = String::from(base);
    for arg in rest {
        out.push(' ');
        out.push_str(arg.as_ref());
    }
    out
}

fn join(base: &str, rest: &[&str]) -> String {
    let mut out = String::from(base);
    for arg in rest {
        out.push(' ');
        out.push_str(arg);
    }
    out
}
