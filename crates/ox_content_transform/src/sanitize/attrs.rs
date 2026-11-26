//! What an attribute's value is checked for before it is written.
//!
//! An attribute either names a resource or holds text, and only the first
//! kind is screened by scheme. The names below cover HTML, SVG and MathML;
//! an attribute added through `allowed_attributes` that they do not cover
//! is screened anyway, because foreign content carries URLs in places no
//! fixed list keeps up with.

use super::config::SanitizeConfig;

/// How a value is screened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    /// One URL.
    Url,
    /// URLs separated by whitespace.
    UrlList,
    /// Candidates of an image source set.
    Srcset,
    /// The `;`-separated values an SVG animation writes into another
    /// attribute, which may be `href`.
    AnimationValues,
    /// Text, never followed as a link.
    Text,
    /// Not known: kept unless it opens like a URL with a disallowed scheme.
    Unknown,
}

fn screen(attr: &str) -> Screen {
    if attr.starts_with("data-") || attr.starts_with("aria-") {
        return Screen::Text;
    }
    match attr {
        "href" | "src" | "action" | "formaction" | "poster" | "xlink:href" | "cite"
        | "background" | "data" | "codebase" | "longdesc" | "manifest" | "icon" | "lowsrc"
        | "dynsrc" | "classid" | "profile" | "to" | "from" | "by" => Screen::Url,
        "ping" | "archive" => Screen::UrlList,
        "srcset" | "imagesrcset" => Screen::Srcset,
        "values" => Screen::AnimationValues,
        "abbr" | "alt" | "class" | "content" | "for" | "id" | "label" | "name" | "pattern"
        | "placeholder" | "role" | "style" | "summary" | "title" | "value" => Screen::Text,
        _ => Screen::Unknown,
    }
}

/// Whether `value`, already decoded, may be written as `attr`.
pub(super) fn allows_value(config: &SanitizeConfig, attr: &str, value: &str) -> bool {
    match screen(attr) {
        Screen::Url => config.allows_url(value),
        Screen::UrlList => value.split_ascii_whitespace().all(|url| config.allows_url(url)),
        Screen::Srcset => config.allows_srcset(value),
        Screen::AnimationValues => value.split(';').all(|part| config.allows_url(part)),
        Screen::Text => true,
        Screen::Unknown => config.allows_scheme_shaped(value),
    }
}

/// Whether `attr` on `tag` would become a named property of `document`.
///
/// `document.<name>` resolves to an `embed`, `form`, `iframe`, `img` or
/// `object` carrying that `name`, and to an `object` carrying that `id`,
/// ahead of the property the page's scripts expect there. Dropping exactly
/// these leaves ids, heading anchors and form fields alone.
pub(super) fn shadows_document_property(tag: &str, attr: &str) -> bool {
    match attr {
        "name" => matches!(tag, "embed" | "form" | "iframe" | "img" | "object"),
        "id" => tag == "object",
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_known_url_attribute_is_screened_as_one() {
        for attr in ["href", "src", "xlink:href", "formaction", "action", "poster", "to"] {
            assert_eq!(screen(attr), Screen::Url, "{attr}");
        }
        assert_eq!(screen("data-href"), Screen::Text);
        assert_eq!(screen("aria-describedby"), Screen::Text);
        assert_eq!(screen("datetime"), Screen::Unknown);
    }
}
