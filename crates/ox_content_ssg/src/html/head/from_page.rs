//! Build [`HeadInput`] from themed pages and bare metadata.

use super::input::{HeadAlternate, HeadInput, HeadJsonLd, SiteHead};
use crate::html::BarePageData;
use crate::html::page::{PageData, SsgConfig};
use crate::html::urls::{absolute_href, page_absolute_url};

/// Themed default: OG/Twitter always, no canonical unless SEO fields are set.
pub fn themed_head_input(page: &PageData, config: &SsgConfig, json_ld: Option<&str>) -> HeadInput {
    let mut input = HeadInput {
        site: SiteHead {
            name: Some(config.site_name.clone()),
            url: config.site_url.clone(),
            locale: config.locale.clone(),
            title_template: None,
        },
        title: Some(page.title.clone()),
        title_suffix: true,
        description: page.description.clone(),
        og_image: config.og_image.clone(),
        social: true,
        emit_site_name: false,
        trusted: true,
        validation: config.head_validation,
        ..HeadInput::default()
    };
    apply_seo(&mut input, page, config);
    if let Some(json) = json_ld.map(str::trim).filter(|json| !json.is_empty()) {
        input.json_ld.push(HeadJsonLd { key: Some("ox:json-ld".into()), json: json.to_string() });
    }
    input
}

/// Bare metadata: social tags only when there is something to say.
pub fn bare_head_input(data: &BarePageData<'_>) -> HeadInput {
    let has_metadata = data.description.is_some()
        || data.canonical_url.is_some()
        || data.site_name.is_some()
        || data.og_image.is_some();
    HeadInput {
        site: SiteHead { name: data.site_name.map(str::to_string), ..SiteHead::default() },
        title: Some(data.title.to_string()),
        title_suffix: false,
        description: data.description.map(str::to_string),
        canonical: data.canonical_url.map(str::to_string),
        og_image: data.og_image.map(str::to_string),
        social: has_metadata,
        emit_site_name: has_metadata,
        trusted: true,
        ..HeadInput::default()
    }
}

fn apply_seo(input: &mut HeadInput, page: &PageData, config: &SsgConfig) {
    input.robots.clone_from(&page.robots);
    input.canonical = page.canonical.clone().or_else(|| {
        config
            .site_url
            .as_deref()
            .and_then(|site| page_absolute_url(site, &config.base, &page.path))
    });
    input.alternates = locale_alternates(config);
}

fn locale_alternates(config: &SsgConfig) -> Vec<HeadAlternate> {
    let Some(site) = config.site_url.as_deref() else {
        return config
            .locale_paths
            .iter()
            .filter_map(|path| {
                let href = path.href.as_deref().or(path.root.as_deref())?.trim();
                if href.starts_with("http://") || href.starts_with("https://") {
                    Some(HeadAlternate { lang: path.code.clone(), href: href.to_string() })
                } else {
                    None
                }
            })
            .collect();
    };
    config
        .locale_paths
        .iter()
        .filter_map(|path| {
            let href = path.href.as_deref().or(path.root.as_deref())?;
            let href = absolute_href(site, href)?;
            Some(HeadAlternate { lang: path.code.clone(), href })
        })
        .collect()
}
