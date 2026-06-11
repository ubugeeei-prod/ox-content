use rustc_hash::FxHashSet;

use crate::JsSanitizeOptions;

pub(super) struct SanitizeConfig {
    tags: FxHashSet<String>,
    attributes: FxHashSet<String>,
    url_schemes: FxHashSet<String>,
}

impl Default for SanitizeConfig {
    fn default() -> Self {
        Self {
            tags: [
                "a",
                "audio",
                "blockquote",
                "br",
                "code",
                "del",
                "details",
                "div",
                "em",
                "h1",
                "h2",
                "h3",
                "h4",
                "h5",
                "h6",
                "hr",
                "iframe",
                "img",
                "input",
                "li",
                "nav",
                "ol",
                "p",
                "picture",
                "pre",
                "source",
                "span",
                "strong",
                "summary",
                "sup",
                "table",
                "tbody",
                "td",
                "th",
                "thead",
                "track",
                "tr",
                "ul",
                "video",
            ]
            .into_iter()
            .map(ToString::to_string)
            .collect(),
            attributes: [
                "allow",
                "allowfullscreen",
                "alt",
                "aria-label",
                "autoplay",
                "checked",
                "class",
                "controls",
                "controlslist",
                "crossorigin",
                "data-code-title",
                "data-group",
                "data-line",
                "data-line-number",
                "data-line-number-start",
                "data-line-numbers",
                "data-ox-tab-group",
                "default",
                "disabled",
                "disablepictureinpicture",
                "disableremoteplayback",
                "height",
                "href",
                "id",
                "kind",
                "label",
                "loading",
                "loop",
                "media",
                "muted",
                "name",
                "playsinline",
                "poster",
                "preload",
                "referrerpolicy",
                "rel",
                "sandbox",
                "sizes",
                "src",
                "srcset",
                "srclang",
                "style",
                "target",
                "title",
                "type",
                "width",
            ]
            .into_iter()
            .map(ToString::to_string)
            .collect(),
            url_schemes: ["http", "https", "mailto", "tel"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
        }
    }
}

impl SanitizeConfig {
    pub(super) fn from_js(options: &JsSanitizeOptions) -> Self {
        let mut config = Self::default();
        if let Some(tags) = &options.allowed_tags {
            config.tags = tags.iter().map(|value| value.to_ascii_lowercase()).collect();
        }
        if let Some(attrs) = &options.allowed_attributes {
            config.attributes = attrs.iter().map(|value| value.to_ascii_lowercase()).collect();
        }
        if let Some(schemes) = &options.allowed_url_schemes {
            config.url_schemes = schemes.iter().map(|value| value.to_ascii_lowercase()).collect();
        }
        config
    }

    pub(super) fn allows_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    pub(super) fn allows_attr(&self, attr: &str) -> bool {
        if attr.starts_with("data-") || attr.starts_with("aria-") {
            return self.attributes.contains("data-*")
                || self.attributes.contains("aria-*")
                || self.attributes.contains(attr);
        }
        self.attributes.contains(attr)
    }

    pub(super) fn allows_url(&self, value: &str) -> bool {
        let trimmed = value.trim_matches(|ch: char| ch.is_ascii_control() || ch.is_whitespace());
        if trimmed.is_empty()
            || trimmed.starts_with('/')
            || trimmed.starts_with("./")
            || trimmed.starts_with("../")
            || trimmed.starts_with('#')
        {
            return true;
        }
        let Some(colon) = trimmed.find(':') else {
            return true;
        };
        let first_path_marker = trimmed.find(&['/', '?', '#'][..]).unwrap_or(usize::MAX);
        if first_path_marker < colon {
            return true;
        }
        let scheme = trimmed[..colon]
            .chars()
            .filter(|ch| !ch.is_ascii_whitespace())
            .flat_map(char::to_lowercase)
            .collect::<String>();
        self.url_schemes.contains(&scheme)
    }

    pub(super) fn allows_srcset(&self, value: &str) -> bool {
        value.split(',').all(|candidate| {
            let candidate = candidate.trim();
            if candidate.is_empty() {
                return false;
            }
            let url_end =
                candidate.find(|ch: char| ch.is_ascii_whitespace()).unwrap_or(candidate.len());
            let url = &candidate[..url_end];
            !url.is_empty() && self.allows_url(url)
        })
    }
}
