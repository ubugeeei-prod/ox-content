//! Edit-URL shapes for the forges a documentation site is hosted on.
//!
//! Every forge exposes a web editor, and no two agree on the path. GitLab
//! puts a `/-/` scope separator in front of it, Bitbucket edits through its
//! source view, and Gitea and Forgejo use `_edit`. A site on any of them
//! needs the right shape or the link 404s.

/// Placeholders a pattern may use.
const REPO_URL: &str = "{repoUrl}";
const BRANCH: &str = "{branch}";
const PATH: &str = "{path}";

const GITHUB: &str = "{repoUrl}/edit/{branch}/{path}";
const GITLAB: &str = "{repoUrl}/-/edit/{branch}/{path}";
const BITBUCKET: &str = "{repoUrl}/src/{branch}/{path}?mode=edit";
const GITEA: &str = "{repoUrl}/_edit/{branch}/{path}";

/// The pattern to build edit URLs with.
///
/// An explicit pattern wins, then a named provider, then the shape implied
/// by the repository host. GitHub is the fallback because it is the shape
/// this option has always produced.
pub(in crate::features) fn resolve_pattern(
    url_pattern: Option<&str>,
    provider: Option<&str>,
    repo_url: &str,
) -> String {
    if let Some(pattern) = url_pattern.map(str::trim).filter(|pattern| !pattern.is_empty()) {
        return pattern.to_string();
    }
    named_pattern(provider.map(str::trim).unwrap_or_default())
        .unwrap_or_else(|| infer_pattern(repo_url))
        .to_string()
}

fn named_pattern(provider: &str) -> Option<&'static str> {
    match provider.to_ascii_lowercase().as_str() {
        "github" => Some(GITHUB),
        "gitlab" => Some(GITLAB),
        "bitbucket" => Some(BITBUCKET),
        "gitea" | "forgejo" => Some(GITEA),
        _ => None,
    }
}

/// The shape implied by the repository host.
///
/// Only the hosted services are recognized. A self-hosted instance keeps
/// GitHub's shape unless `provider` or a pattern says otherwise, because
/// its hostname says nothing about the software behind it.
fn infer_pattern(repo_url: &str) -> &'static str {
    let host = host_of(repo_url).to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host);
    match host {
        "gitlab.com" => GITLAB,
        "bitbucket.org" => BITBUCKET,
        "codeberg.org" | "gitea.com" => GITEA,
        _ => GITHUB,
    }
}

fn host_of(repo_url: &str) -> &str {
    let after_scheme = repo_url.split_once("://").map_or(repo_url, |(_, rest)| rest);
    let authority = after_scheme.split(['/', '?', '#']).next().unwrap_or_default();
    let host = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
    host.split(':').next().unwrap_or_default()
}

/// Fills `pattern` in, leaving unknown placeholders as written.
pub(in crate::features) fn render_pattern(
    pattern: &str,
    repo_url: &str,
    branch: &str,
    path: &str,
) -> String {
    let mut out = String::with_capacity(pattern.len() + repo_url.len() + path.len());
    let mut rest = pattern;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let tail = &rest[open..];
        let replacement = if tail.starts_with(REPO_URL) {
            Some((repo_url, REPO_URL.len()))
        } else if tail.starts_with(BRANCH) {
            Some((branch, BRANCH.len()))
        } else if tail.starts_with(PATH) {
            Some((path, PATH.len()))
        } else {
            None
        };
        if let Some((value, len)) = replacement {
            out.push_str(value);
            rest = &tail[len..];
        } else {
            out.push('{');
            rest = &tail[1..];
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests;
