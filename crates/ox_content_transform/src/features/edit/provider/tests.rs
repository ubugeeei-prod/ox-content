use super::{render_pattern, resolve_pattern};

fn url(provider: Option<&str>, repo_url: &str) -> String {
    render_pattern(&resolve_pattern(None, provider, repo_url), repo_url, "main", "docs/guide.md")
}

#[test]
fn each_named_provider_has_its_own_shape() {
    assert_eq!(
        url(Some("github"), "https://github.com/owner/repo"),
        "https://github.com/owner/repo/edit/main/docs/guide.md"
    );
    assert_eq!(
        url(Some("gitlab"), "https://git.example.com/owner/repo"),
        "https://git.example.com/owner/repo/-/edit/main/docs/guide.md"
    );
    assert_eq!(
        url(Some("bitbucket"), "https://git.example.com/owner/repo"),
        "https://git.example.com/owner/repo/src/main/docs/guide.md?mode=edit"
    );
    assert_eq!(
        url(Some("gitea"), "https://git.example.com/owner/repo"),
        "https://git.example.com/owner/repo/_edit/main/docs/guide.md"
    );
    assert_eq!(
        url(Some("forgejo"), "https://git.example.com/o/r"),
        url(Some("gitea"), "https://git.example.com/o/r")
    );
}

#[test]
fn a_named_provider_is_case_insensitive() {
    assert_eq!(
        url(Some("GitLab"), "https://github.com/owner/repo"),
        "https://github.com/owner/repo/-/edit/main/docs/guide.md"
    );
}

#[test]
fn hosted_forges_are_inferred_from_the_repository_url() {
    assert_eq!(
        url(None, "https://gitlab.com/owner/repo"),
        "https://gitlab.com/owner/repo/-/edit/main/docs/guide.md"
    );
    assert_eq!(
        url(None, "https://bitbucket.org/owner/repo"),
        "https://bitbucket.org/owner/repo/src/main/docs/guide.md?mode=edit"
    );
    assert_eq!(
        url(None, "https://codeberg.org/owner/repo"),
        "https://codeberg.org/owner/repo/_edit/main/docs/guide.md"
    );
}

#[test]
fn an_unknown_host_or_provider_keeps_the_github_shape() {
    let expected = "https://git.example.com/owner/repo/edit/main/docs/guide.md";

    assert_eq!(url(None, "https://git.example.com/owner/repo"), expected);
    assert_eq!(url(Some("sourcehut"), "https://git.example.com/owner/repo"), expected);
    assert_eq!(url(Some(""), "https://git.example.com/owner/repo"), expected);
}

#[test]
fn inference_reads_the_host_and_not_the_rest_of_the_url() {
    // A path or a userinfo section that merely mentions another forge must
    // not change the shape.
    assert_eq!(
        url(None, "https://github.com/owner/gitlab.com"),
        "https://github.com/owner/gitlab.com/edit/main/docs/guide.md"
    );
    assert_eq!(
        url(None, "https://user@gitlab.com:8443/owner/repo"),
        "https://user@gitlab.com:8443/owner/repo/-/edit/main/docs/guide.md"
    );
    assert_eq!(
        url(None, "https://WWW.GitLab.com/owner/repo"),
        "https://WWW.GitLab.com/owner/repo/-/edit/main/docs/guide.md"
    );
}

#[test]
fn an_explicit_pattern_wins_over_the_provider_and_the_host() {
    let pattern = "{repoUrl}/ui/edit?ref={branch}&file={path}";
    let resolved = resolve_pattern(Some(pattern), Some("gitlab"), "https://gitlab.com/o/r");

    assert_eq!(
        render_pattern(&resolved, "https://gitlab.com/o/r", "trunk", "docs/guide.md"),
        "https://gitlab.com/o/r/ui/edit?ref=trunk&file=docs/guide.md"
    );
}

#[test]
fn a_blank_pattern_falls_back_to_the_provider() {
    assert_eq!(
        render_pattern(
            &resolve_pattern(Some("   "), Some("gitlab"), "https://github.com/o/r"),
            "https://github.com/o/r",
            "main",
            "docs/guide.md",
        ),
        "https://github.com/o/r/-/edit/main/docs/guide.md"
    );
}

#[test]
fn braces_that_are_not_placeholders_stay_literal() {
    assert_eq!(
        render_pattern("{repoUrl}/{unknown}/{{/{path}", "https://x/y", "main", "a.md"),
        "https://x/y/{unknown}/{{/a.md"
    );
}
