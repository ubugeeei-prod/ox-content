use super::html::{ComponentElement, attr};
use super::provider_cards::{
    Card, body_text, first_attr, host_in, parse_https_url, provider_url, render_card,
};

pub(super) fn render_npm_package(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let reference = package_reference(href, Registry::Npm)?;
    Some(render_package_card(element, href, reference))
}

pub(super) fn render_crates_io(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let reference = package_reference(href, Registry::CratesIo)?;
    Some(render_package_card(element, href, reference))
}

pub(super) fn render_pypi(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let reference = package_reference(href, Registry::Pypi)?;
    Some(render_package_card(element, href, reference))
}

pub(super) fn render_docker_hub(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let reference = package_reference(href, Registry::DockerHub)?;
    Some(render_package_card(element, href, reference))
}

enum Registry {
    Npm,
    CratesIo,
    Pypi,
    DockerHub,
}

struct PackageReference {
    modifier: &'static str,
    network: &'static str,
    name: String,
    version: Option<String>,
}

fn render_package_card(
    element: &ComponentElement<'_>,
    href: &str,
    reference: PackageReference,
) -> String {
    let version = first_attr(element, &["version", "tag"]).or(reference.version.as_deref());
    let title = first_attr(element, &["title", "name", "package", "packageName"])
        .unwrap_or(reference.name.as_str());
    let repository = first_attr(element, &["repository", "repo"]);
    let downloads = first_attr(element, &["downloads", "downloadCount", "pulls", "pullCount"]);

    render_card(Card {
        modifier: reference.modifier,
        network: reference.network,
        href,
        title,
        body: body_text(element)
            .or_else(|| attr(element, "description"))
            .or_else(|| attr(element, "excerpt")),
        source_label: "Open package",
        image: None,
        avatar: None,
        author: first_attr(element, &["author", "owner", "maintainer"]),
        date: first_attr(element, &["dateTime", "updatedAt", "updated", "publishedAt"]),
        date_label: first_attr(element, &["dateLabel", "updatedLabel", "publishedAtLabel"]),
        meta: vec![
            ("Version", version),
            ("License", attr(element, "license")),
            ("Repository", repository),
            ("Downloads", downloads),
            ("Stars", first_attr(element, &["stars", "starCount"])),
        ],
        iframe: None,
    })
}

fn package_reference(input: &str, registry: Registry) -> Option<PackageReference> {
    let parsed = parse_https_url(input)?;
    let segments = path_segments(parsed.path);
    match registry {
        Registry::Npm => npm_reference(&parsed.host, &segments),
        Registry::CratesIo => crates_reference(&parsed.host, &segments),
        Registry::Pypi => pypi_reference(&parsed.host, &segments),
        Registry::DockerHub => docker_reference(&parsed.host, &segments, parsed.query),
    }
}

fn npm_reference(host: &str, segments: &[&str]) -> Option<PackageReference> {
    if !host_in(host, &["www.npmjs.com", "npmjs.com"]) || segments.first()? != &"package" {
        return None;
    }
    let (name, version_index) = if segments.get(1)?.starts_with('@') {
        let scope = safe_npm_part(segments.get(1)?)?;
        let package = safe_npm_part(segments.get(2)?)?;
        (format!("{scope}/{package}"), 3)
    } else {
        (safe_npm_part(segments.get(1)?)?.to_string(), 2)
    };
    Some(PackageReference {
        modifier: "npm",
        network: "npm",
        name,
        version: version_segment(segments, version_index),
    })
}

fn crates_reference(host: &str, segments: &[&str]) -> Option<PackageReference> {
    if !host_in(host, &["crates.io"]) || segments.first()? != &"crates" {
        return None;
    }
    Some(PackageReference {
        modifier: "crates-io",
        network: "crates.io",
        name: safe_registry_name(segments.get(1)?)?.to_string(),
        version: segments.get(2).copied().filter(|value| safe_version(value)).map(str::to_string),
    })
}

fn pypi_reference(host: &str, segments: &[&str]) -> Option<PackageReference> {
    if !host_in(host, &["pypi.org"]) || segments.first()? != &"project" {
        return None;
    }
    Some(PackageReference {
        modifier: "pypi",
        network: "PyPI",
        name: safe_registry_name(segments.get(1)?)?.to_string(),
        version: segments.get(2).copied().filter(|value| safe_version(value)).map(str::to_string),
    })
}

fn docker_reference(
    host: &str,
    segments: &[&str],
    query: Option<&str>,
) -> Option<PackageReference> {
    if !host_in(host, &["hub.docker.com"]) {
        return None;
    }
    let (namespace, repository, rest) = match segments {
        ["_", repository, rest @ ..] => ("library", *repository, rest),
        ["r", namespace, repository, rest @ ..] => (*namespace, *repository, rest),
        ["repository", "docker", namespace, repository, rest @ ..] => {
            (*namespace, *repository, rest)
        }
        _ => return None,
    };
    let namespace = safe_registry_name(namespace)?;
    let repository = safe_registry_name(repository)?;
    Some(PackageReference {
        modifier: "docker-hub",
        network: "Docker Hub",
        name: format!("{namespace}/{repository}"),
        version: docker_tag(rest, query),
    })
}

fn docker_tag(rest: &[&str], query: Option<&str>) -> Option<String> {
    if rest.first() == Some(&"tags")
        && let Some(tag) = rest.get(1).copied().filter(|value| safe_version(value))
    {
        return Some(tag.to_string());
    }
    query_param(query?, "name").filter(|value| safe_version(value)).map(str::to_string)
}

fn query_param<'a>(query: &'a str, name: &str) -> Option<&'a str> {
    query.split('&').find_map(|part| {
        let (key, value) = part.split_once('=')?;
        (key == name && !value.is_empty()).then_some(value)
    })
}

fn version_segment(segments: &[&str], index: usize) -> Option<String> {
    if segments.get(index) == Some(&"v") {
        return segments
            .get(index + 1)
            .copied()
            .filter(|value| safe_version(value))
            .map(str::to_string);
    }
    None
}

fn path_segments(path: &str) -> Vec<&str> {
    path.split('/').filter(|segment| !segment.is_empty()).collect()
}

fn safe_npm_part(value: &str) -> Option<&str> {
    (!value.contains("..")
        && value.len() <= 214
        && value.chars().all(|ch| {
            ch.is_ascii_alphanumeric() || matches!(ch, '@' | '-' | '_' | '.' | '~' | '%' | '+')
        }))
    .then_some(value)
}

fn safe_registry_name(value: &str) -> Option<&str> {
    (value.len() <= 128
        && value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.')))
    .then_some(value)
}

fn safe_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '+' | ':' | '@'))
}
