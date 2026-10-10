//! `oxct tui` without a terminal prints the rendered document and exits.

#[path = "support/oxct.rs"]
mod oxct;

use oxct::Project;
use unicode_width::UnicodeWidthStr;

const RICH: &str = "---\ntitle: Sample\n---\n\n# 日本語の見出し 👩‍💻\n\nA **bold** paragraph with *emphasis*, `code`, and [a guide](./guide.md) that keeps going well past any narrow terminal width.\n\n> A quote with a https://example.com/a/very/long/url/that/cannot/be/broken/at/a/space\n\n1. First\n2. Second\n   - Nested [x]\n\n```ts\nconst greeting = \"こんにちは、世界\";\n```\n\n| Name | Value | Notes |\n| --- | --- | --- |\n| 日本語 | 42 | A longer cell that wraps |\n";

fn project() -> Project {
    let project = Project::new();
    project.write("b.md", "# B doc\n");
    project.write("docs/a.md", "# A doc\n\ntext\n");
    project.write("docs/z.txt", "# Z text\n");
    project
}

#[track_caller]
fn print(project: &Project, args: &[&str]) -> String {
    let run = project.run(&[&["tui"], args].concat());
    run.success();
    assert_eq!(run.stderr, "");
    run.stdout
}

#[track_caller]
fn render(source: &str, args: &[&str]) -> String {
    let run = Project::new().run_with(&[&["tui", "--stdin"], args].concat(), source.as_bytes());
    run.success();
    run.stdout
}

#[test]
fn a_directory_prints_its_first_document_in_path_order() {
    let project = project();
    let rule = "─".repeat(40);
    let width = ["--width", "40"];
    assert_eq!(print(&project, &width), format!("# B doc\n{rule}\n\n"));
    assert_eq!(print(&project, &[&["."], &width[..]].concat()), print(&project, &width));
    let docs = print(&project, &[&["docs"], &width[..]].concat());
    assert_eq!(docs, format!("# A doc\n{rule}\n\ntext\n\n"));
    assert_eq!(print(&project, &[&["docs/*.md"], &width[..]].concat()), docs);
}

#[test]
fn a_named_file_is_printed_whatever_its_extension() {
    let project = project();
    assert!(print(&project, &["docs/a.md"]).starts_with("# A doc\n"));
    assert!(print(&project, &["docs/z.txt"]).starts_with("# Z text\n"));
    let absolute = project.file("b.md");
    assert!(print(&project, &[absolute.to_str().unwrap()]).starts_with("# B doc\n"));
}

#[test]
fn print_watch_and_colour_flags_do_not_change_piped_output() {
    let project = project();
    let plain = print(&project, &["b.md"]);
    for flags in [
        &["--print"][..],
        &["--no-watch"],
        &["--no-color"],
        &["--print", "--no-watch", "--no-color"],
        &["--theme", "nord"],
        &["--theme", "light"],
        &["--theme", "mono"],
    ] {
        assert_eq!(print(&project, &[&["b.md"], flags].concat()), plain, "{flags:?}");
    }
    assert!(!plain.contains('\x1b'));
}

#[test]
fn every_printed_line_fits_the_requested_width() {
    for width in [20, 21, 33, 40, 64, 80, 120, 240] {
        let output = render(RICH, &["--width", &width.to_string()]);
        for line in output.lines() {
            assert!(line.width() <= width, "--width {width}: {line:?} is {} wide", line.width());
        }
        let rule = output.lines().find(|line| line.starts_with('─')).unwrap();
        assert_eq!(rule.width(), width, "the h1 rule spans the page");
        for expected in ["◇ Frontmatter", "title: Sample", "╭─ ts", "1. First", "│ A quote"]
        {
            assert!(output.contains(expected), "--width {width} lost {expected:?}");
        }
    }
}

/// Without `--width` the page follows the attached console, or 80 columns when there is none.
#[test]
fn default_width_stays_within_the_supported_range() {
    let output = render("# Title\n", &[]);
    let rule = output.lines().nth(1).unwrap();
    assert!(rule.chars().all(|ch| ch == '─'), "{output}");
    assert!((20..=240).contains(&rule.width()), "{} columns", rule.width());
    assert_eq!(render("# Title\n", &["--width", "80"]), format!("# Title\n{}\n\n", "─".repeat(80)));
}

#[test]
fn stdin_documents_render_identically_to_files() {
    let project = Project::new();
    project.write("rich.md", RICH);
    assert_eq!(print(&project, &["rich.md", "--width", "60"]), render(RICH, &["--width", "60"]));
}

#[test]
fn line_endings_tabs_and_controls_are_normalised() {
    let unix =
        render("# Title\n\nfirst line\nsecond line\n\n```\n\tcode\n```\n", &["--width", "40"]);
    let windows = render(
        "# Title\r\n\r\nfirst line\r\nsecond line\r\n\r\n```\r\n\tcode\r\n```\r\n",
        &["--width", "40"],
    );
    assert_eq!(unix, windows);
    assert!(unix.contains("first line second line\n"), "{unix}");
    assert!(unix.contains("│     code\n"), "{unix}");
    let hostile = render(
        "# Safe\x1b[2J\x07\u{9b}31m\n\ntext\x1b]0;title\x07 \x00end\x7f\n",
        &["--width", "40"],
    );
    assert!(!hostile.chars().any(|ch| ch.is_control() && ch != '\n'), "{hostile:?}");
    assert!(hostile.starts_with("# Safe[2J31m\n"), "{hostile:?}");
}

#[test]
fn raw_html_is_dropped_but_its_text_stays_readable() {
    let output = render(
        "Before <b>bold</b> after.\n\n<script>alert(1)</script>\n\n<div>\nblock\n</div>\n",
        &["--width", "40"],
    );
    assert!(output.starts_with("Before bold after.\n"), "{output}");
    assert!(!output.contains('<') && !output.contains("alert"), "{output}");
}

#[test]
fn empty_and_whitespace_documents_print_nothing() {
    for source in ["", "\n", "\n\n\n", "   \n\t\n"] {
        assert_eq!(render(source, &[]), "", "{source:?}");
    }
}

#[test]
fn invalid_arguments_are_rejected_with_the_accepted_range() {
    let project = project();
    for (args, reason) in [
        (&["--width", "19"][..], "19 is not in 20..=240"),
        (&["--width", "241"], "241 is not in 20..=240"),
        (&["--width", "0"], "0 is not in 20..=240"),
        (&["--width", "-5"], "unexpected argument '-5'"),
        (&["--width", "wide"], "invalid value 'wide'"),
        (&["--theme", "dark"], "[possible values: nord, light, mono]"),
        (&["--theme", "Nord"], "invalid value 'Nord'"),
        (&["--stdin", "b.md"], "'--stdin' cannot be used with '[PATH]'"),
        (&["b.md", "docs/a.md"], "unexpected argument 'docs/a.md'"),
        (&["--fix"], "unexpected argument '--fix'"),
    ] {
        project.run(&[&["tui"], args].concat()).rejected(reason);
    }
    for width in ["20", "240"] {
        project.run(&["tui", "b.md", "--width", width]).success();
    }
}

#[test]
fn missing_inputs_fail_with_an_actionable_message() {
    let project = project();
    std::fs::create_dir(project.file("empty")).unwrap();
    for input in ["missing.md", "missing/", "empty", "docs/*.mdx", "node_modules"] {
        project.run(&["tui", input]).rejected("No Markdown files matched the viewer input");
    }
    project.run(&["tui", "docs/["]).rejected("error parsing glob");
}

#[test]
fn oversized_and_binary_documents_are_refused() {
    let project = Project::new();
    let oversized = vec![b'a'; 4 * 1024 * 1024 + 1];
    project
        .run_with(&["tui", "--stdin"], &oversized)
        .rejected("Document exceeds the 4 MiB viewer limit");
    std::fs::write(project.file("big.md"), &oversized).unwrap();
    project.run(&["tui", "big.md"]).rejected("Document exceeds the 4 MiB viewer limit");
    project.run_with(&["tui", "--stdin"], b"# Title\n\xff\xfe").rejected("invalid utf-8");
    std::fs::write(project.file("binary.md"), b"\x89PNG\r\n\x1a\n\xff").unwrap();
    project.run(&["tui", "binary.md"]).rejected("invalid utf-8");
}

#[test]
fn viewer_ignores_lint_configuration_and_ignore_files() {
    let project = project();
    project.write(".markdownlintignore", "*.md\n");
    project.write(".oxlint.json", "{ not json");
    assert!(print(&project, &[]).starts_with("# B doc\n"));
}
