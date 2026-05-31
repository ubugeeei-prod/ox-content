//! Package-manager install tabs transform.
//!
//! Authors write a single npm-style command inside a `<pm>` element, e.g.
//!
//! ```html
//! <pm>npm install -D vite</pm>
//! ```
//!
//! and the post-render transform expands it into the same accessible, no-JS tab
//! widget produced by [`crate::tabs`], with one tab per package manager
//! (npm/pnpm/yarn/bun, in that order). Each tab body is a code block containing
//! the command converted to that package manager's equivalent.
//!
//! The conversion is implemented natively here (no shell-out, no JS). It covers
//! the common install/run/exec verbs; unrecognized commands are passed through
//! unchanged so authoring never silently breaks.
//!
//! Like the generic tabs transform this operates on already-rendered HTML and
//! reuses the exact `ox-tabs` markup so styling and keyboard navigation stay
//! consistent. When `sync` is enabled a `data-ox-tab-group` attribute is emitted
//! on the group element so the client runtime can keep groups in sync via
//! `localStorage`; when disabled the attribute is omitted and the output matches
//! the generic widget byte-for-byte (modulo the converted command bodies).

use std::fmt::Write;

use crate::html_scan::find_ci;

/// The package managers we expand to, in display order.
const PACKAGE_MANAGERS: [&str; 4] = ["npm", "pnpm", "yarn", "bun"];

/// The natural group key used for synced package-manager tabs.
pub const PM_GROUP_KEY: &str = "pkg-manager";

/// Options for [`transform_pm`].
#[derive(Debug, Clone, Copy, Default)]
pub struct PmOptions {
    /// When `true`, emit a `data-ox-tab-group` attribute so the client runtime
    /// syncs the active package manager across every pm tab group on the page.
    /// Off by default.
    pub sync: bool,
}

/// Result of [`transform_pm`]: rewritten HTML and the number of `<pm>` groups
/// expanded (so the caller can advance its shared tab-group counter).
pub struct PmTransform {
    pub html: String,
    pub group_count: u32,
}

/// Expand every `<pm>` block in `html`, numbering groups from `start_group`.
pub fn transform_pm(html: &str, start_group: u32, options: PmOptions) -> PmTransform {
    if find_ci(html, 0, "<pm").is_none() {
        return PmTransform { html: html.to_string(), group_count: 0 };
    }

    let mut out = String::with_capacity(html.len());
    let mut cursor = 0;
    let mut next_group = start_group;

    while let Some(open_at) = find_tag(html, cursor, "<pm") {
        out.push_str(&html[cursor..open_at]);

        let Some(tag) = scan_start_tag(html, open_at) else {
            out.push('<');
            cursor = open_at + 1;
            continue;
        };

        if tag.self_closing {
            out.push_str(&html[open_at..tag.end]);
            cursor = tag.end;
            continue;
        }

        let Some(close_start) = find_matching_close(html, tag.end, "<pm", "</pm>") else {
            out.push_str(&html[open_at..tag.end]);
            cursor = tag.end;
            continue;
        };
        let inner = &html[tag.end..close_start];
        let block_end = close_start + "</pm>".len();

        let command = extract_command(inner);
        if command.is_empty() {
            out.push_str(&html[open_at..block_end]);
        } else {
            out.push_str(&render_pm(&command, next_group, options));
            next_group += 1;
        }
        cursor = block_end;
    }

    out.push_str(&html[cursor..]);
    PmTransform { html: out, group_count: next_group - start_group }
}

/// Pull the bare npm command out of a `<pm>` element's inner HTML.
///
/// The element may contain plain text (`<pm>npm i vite</pm>`) or a rendered code
/// block (`<pm><pre><code>npm i vite</code></pre></pm>`); both are supported. We
/// strip any tags, decode the handful of entities a renderer can introduce, and
/// collapse whitespace.
fn extract_command(inner: &str) -> String {
    let mut text = String::with_capacity(inner.len());
    let mut in_tag = false;
    for ch in inner.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }
    let decoded = decode_entities(&text);
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Decode the small set of HTML entities a renderer may emit inside code.
fn decode_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
}

/// Render the expanded package-manager tab widget.
///
/// This mirrors [`crate::tabs`] output exactly (same classes, ARIA-free CSS
/// `:has()` widget, `<noscript>` `<details>` fallback) so the shared tab CSS and
/// keyboard runtime apply unchanged. The only additions are the converted code
/// bodies and the opt-in `data-ox-tab-group` attribute.
fn render_pm(command: &str, group: u32, options: PmOptions) -> String {
    let group_str = group.to_string();
    let commands: Vec<String> =
        PACKAGE_MANAGERS.iter().map(|pm| convert_command(command, pm)).collect();

    let mut out = String::new();
    out.push_str("<div class=\"ox-tabs-container\"><div class=\"ox-tabs\" data-group=\"");
    out.push_str(&group_str);
    out.push('"');
    if options.sync {
        out.push_str(" data-ox-tab-group=\"");
        out.push_str(PM_GROUP_KEY);
        out.push('"');
    }
    out.push_str("><div class=\"ox-tabs-header\">");
    for (index, pm) in PACKAGE_MANAGERS.iter().enumerate() {
        out.push_str("<input type=\"radio\" name=\"ox-tabs-");
        out.push_str(&group_str);
        out.push_str("\" id=\"ox-tab-");
        let _ = write!(out, "{group_str}-{index}");
        out.push('"');
        if index == 0 {
            out.push_str(" checked");
        }
        out.push_str("><label for=\"ox-tab-");
        let _ = write!(out, "{group_str}-{index}");
        out.push_str("\">");
        out.push_str(pm);
        out.push_str("</label>");
    }
    out.push_str("</div>");
    for (index, command) in commands.iter().enumerate() {
        out.push_str("<div class=\"ox-tab-panel\" data-tab=\"");
        let _ = write!(out, "{index}");
        out.push_str("\">");
        out.push_str(&code_block(command));
        out.push_str("</div>");
    }
    out.push_str("</div><noscript><div class=\"ox-tabs-fallback\">");
    for (index, (pm, command)) in PACKAGE_MANAGERS.iter().zip(commands.iter()).enumerate() {
        out.push_str("<details");
        if index == 0 {
            out.push_str(" open");
        }
        out.push_str("><summary>");
        out.push_str(pm);
        out.push_str("</summary><div class=\"ox-tabs-fallback-content\">");
        out.push_str(&code_block(command));
        out.push_str("</div></details>");
    }
    out.push_str("</div></noscript></div>");
    out
}

/// Wrap a converted command in a `<pre><code>` block, escaping its text.
fn code_block(command: &str) -> String {
    let mut out = String::with_capacity(command.len() + 24);
    out.push_str("<pre><code>");
    out.push_str(&escape_html(command));
    out.push_str("</code></pre>");
    out
}

/// Escape HTML special characters in command text.
fn escape_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Convert an npm-style command to the equivalent for `target` (one of
/// `npm`/`pnpm`/`yarn`/`bun`). `npm` returns the command unchanged. Unknown
/// shapes fall back to a best-effort binary swap so nothing silently breaks.
pub fn convert_command(command: &str, target: &str) -> String {
    if target == "npm" {
        return command.to_string();
    }

    let tokens: Vec<&str> = command.split_whitespace().collect();
    if tokens.is_empty() {
        return command.to_string();
    }

    // `npx <bin> …`
    if tokens[0] == "npx" {
        let rest = &tokens[1..];
        return match target {
            "pnpm" => join("pnpm dlx", rest),
            "yarn" => join("yarn dlx", rest),
            "bun" => join("bunx", rest),
            _ => command.to_string(),
        };
    }

    // Everything else is expected to start with `npm`.
    if tokens[0] != "npm" {
        return command.to_string();
    }
    if tokens.len() < 2 {
        // Bare `npm`: just swap the binary name.
        return target.to_string();
    }

    let verb = tokens[1];
    let args = &tokens[2..];

    match verb {
        "install" | "i" | "add" => convert_add(args, target),
        "uninstall" | "remove" | "rm" | "un" => convert_remove(args, target),
        "run" => convert_run(args, target),
        // Lifecycle/passthrough scripts like `npm test`, `npm start`, `npm ci`.
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
    let has_packages = scope.rest.iter().any(|a| !a.starts_with('-'));

    // No packages → install/sync the project (e.g. `npm install`).
    if !has_packages && !scope.global {
        return match target {
            "pnpm" => with_rest("pnpm install", &scope.rest),
            "yarn" => with_rest("yarn", &scope.rest),
            "bun" => with_rest("bun install", &scope.rest),
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
            // yarn (classic) uses `yarn global add` for global installs.
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
        _ => with_rest("npm install", args),
    }
}

fn convert_remove(args: &[&str], target: &str) -> String {
    let scope = classify_scope(args);
    match target {
        "pnpm" => {
            let mut base = String::from("pnpm remove");
            if scope.global {
                base.push_str(" -g");
            }
            with_rest(&base, &scope.rest)
        }
        "yarn" => {
            let mut base = String::from("yarn");
            if scope.global {
                base.push_str(" global remove");
            } else {
                base.push_str(" remove");
            }
            with_rest(&base, &scope.rest)
        }
        "bun" => {
            let mut base = String::from("bun remove");
            if scope.global {
                base.push_str(" -g");
            }
            with_rest(&base, &scope.rest)
        }
        _ => with_rest("npm uninstall", args),
    }
}

fn convert_run(args: &[&str], target: &str) -> String {
    match target {
        // pnpm/yarn/bun all accept `<pm> run <script>`; yarn idiomatically drops
        // `run`, so use `yarn <script>`.
        "pnpm" => with_rest("pnpm run", args),
        "yarn" => with_rest("yarn", args),
        "bun" => with_rest("bun run", args),
        _ => with_rest("npm run", args),
    }
}

fn convert_passthrough(verb: &str, args: &[&str], target: &str) -> String {
    let mut base = String::from(target);
    base.push(' ');
    base.push_str(verb);
    with_rest(&base, args)
}

/// Join a base command with the remaining (string) args.
fn with_rest(base: &str, rest: &[impl AsRef<str>]) -> String {
    let mut out = String::from(base);
    for arg in rest {
        out.push(' ');
        out.push_str(arg.as_ref());
    }
    out
}

/// Join a base command literal with `&str` args.
fn join(base: &str, rest: &[&str]) -> String {
    let mut out = String::from(base);
    for arg in rest {
        out.push(' ');
        out.push_str(arg);
    }
    out
}

// --- HTML scanning helpers (shared shape with `tabs.rs`). ---

struct StartTag {
    end: usize,
    self_closing: bool,
}

fn scan_start_tag(html: &str, pos: usize) -> Option<StartTag> {
    let bytes = html.as_bytes();
    if bytes.get(pos) != Some(&b'<') {
        return None;
    }
    let mut i = pos + 1;
    let mut quote: Option<u8> = None;
    let mut tag_end = None;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == q {
                    quote = None;
                }
            }
            None => {
                if b == b'"' || b == b'\'' {
                    quote = Some(b);
                } else if b == b'>' {
                    tag_end = Some(i);
                    break;
                }
            }
        }
        i += 1;
    }
    let tag_end = tag_end?;
    let self_closing = tag_end > pos && bytes[tag_end - 1] == b'/';
    Some(StartTag { end: tag_end + 1, self_closing })
}

fn find_tag(html: &str, from: usize, open: &str) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut search = from;
    loop {
        let at = find_ci(html, search, open)?;
        let after = at + open.len();
        let boundary = bytes.get(after).copied();
        if matches!(boundary, Some(b) if b == b'>' || b == b'/' || b.is_ascii_whitespace()) {
            return Some(at);
        }
        search = after;
    }
}

fn find_matching_close(html: &str, from: usize, open: &str, close: &str) -> Option<usize> {
    let mut depth = 1usize;
    let mut search = from;
    loop {
        let next_open = find_tag(html, search, open);
        let next_close = find_ci(html, search, close);
        match (next_open, next_close) {
            (Some(open_at), Some(close_at)) if open_at < close_at => {
                depth += 1;
                search = match scan_start_tag(html, open_at) {
                    Some(tag) => tag.end,
                    None => open_at + 1,
                };
            }
            (_, Some(close_at)) => {
                depth -= 1;
                if depth == 0 {
                    return Some(close_at);
                }
                search = close_at + close.len();
            }
            (_, None) => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all(command: &str) -> (String, String, String, String) {
        (
            convert_command(command, "npm"),
            convert_command(command, "pnpm"),
            convert_command(command, "yarn"),
            convert_command(command, "bun"),
        )
    }

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
    fn transform_expands_pm_block() {
        let result = transform_pm("<pm>npm install -D vite</pm>", 0, PmOptions { sync: false });
        assert_eq!(result.group_count, 1);
        assert!(result.html.contains("<label for=\"ox-tab-0-0\">npm</label>"));
        assert!(result.html.contains("<label for=\"ox-tab-0-1\">pnpm</label>"));
        assert!(result.html.contains("<label for=\"ox-tab-0-2\">yarn</label>"));
        assert!(result.html.contains("<label for=\"ox-tab-0-3\">bun</label>"));
        assert!(result.html.contains("<pre><code>npm install -D vite</code></pre>"));
        assert!(result.html.contains("<pre><code>pnpm add -D vite</code></pre>"));
        assert!(result.html.contains("<pre><code>bun add -D vite</code></pre>"));
    }

    #[test]
    fn transform_handles_code_block_inner() {
        let result = transform_pm(
            "<pm><pre><code>npm i vite</code></pre></pm>",
            0,
            PmOptions { sync: false },
        );
        assert!(result.html.contains("<pre><code>pnpm add vite</code></pre>"));
    }

    #[test]
    fn group_attr_only_when_sync_enabled() {
        let off = transform_pm("<pm>npm i vite</pm>", 0, PmOptions { sync: false });
        assert!(!off.html.contains("data-ox-tab-group"));

        let on = transform_pm("<pm>npm i vite</pm>", 0, PmOptions { sync: true });
        assert!(on.html.contains("data-ox-tab-group=\"pkg-manager\""));
    }

    #[test]
    fn numbers_groups_and_passes_through_without_marker() {
        let result = transform_pm("<p>nothing here</p>", 7, PmOptions::default());
        assert_eq!(result.group_count, 0);
        assert_eq!(result.html, "<p>nothing here</p>");
    }

    #[test]
    fn numbers_multiple_groups_from_start() {
        let result = transform_pm("<pm>npm i a</pm> mid <pm>npm i b</pm>", 3, PmOptions::default());
        assert_eq!(result.group_count, 2);
        assert!(result.html.contains("data-group=\"3\""));
        assert!(result.html.contains("data-group=\"4\""));
        assert!(result.html.contains(" mid "));
    }

    #[test]
    fn empty_pm_block_left_untouched() {
        let html = "<pm>   </pm>";
        let result = transform_pm(html, 0, PmOptions::default());
        assert_eq!(result.group_count, 0);
        assert_eq!(result.html, html);
    }
}
