//! VitePress migration helpers.

use serde_json::{Map, Number, Value};

/// Normalizes VitePress frontmatter into the ox-content entry-page shape.
pub fn normalize_vitepress_frontmatter(frontmatter: Value) -> Value {
    let Value::Object(mut next) = frontmatter else {
        return frontmatter;
    };

    if next.get("layout").and_then(Value::as_str) == Some("home") {
        next.insert("layout".to_string(), Value::String("entry".to_string()));
    }

    if let Some(Value::Object(hero)) = next.get("hero") {
        if let Some(image) = hero.get("image").and_then(normalize_hero_image) {
            let mut next_hero = hero.clone();
            next_hero.insert("image".to_string(), image);
            next.insert("hero".to_string(), Value::Object(next_hero));
        }
    }

    Value::Object(next)
}

fn normalize_hero_image(value: &Value) -> Option<Value> {
    if let Some(src) = value.as_str() {
        return Some(Value::Object(Map::from_iter([(
            "src".to_string(),
            Value::String(src.to_string()),
        )])));
    }

    let Value::Object(image) = value else {
        return None;
    };

    let src = non_empty_string(image, "src")
        .or_else(|| non_empty_string(image, "light"))
        .or_else(|| non_empty_string(image, "dark"))?;

    let mut normalized = Map::new();
    normalized.insert("src".to_string(), Value::String(src.to_string()));

    if let Some(light_src) =
        non_empty_string(image, "lightSrc").or_else(|| non_empty_string(image, "light"))
    {
        normalized.insert("lightSrc".to_string(), Value::String(light_src.to_string()));
    }

    if let Some(dark_src) =
        non_empty_string(image, "darkSrc").or_else(|| non_empty_string(image, "dark"))
    {
        normalized.insert("darkSrc".to_string(), Value::String(dark_src.to_string()));
    }

    if let Some(alt) = image.get("alt").and_then(Value::as_str) {
        normalized.insert("alt".to_string(), Value::String(alt.to_string()));
    }

    if let Some(width) = to_number(image.get("width")) {
        normalized.insert("width".to_string(), Value::Number(width));
    }

    if let Some(height) = to_number(image.get("height")) {
        normalized.insert("height".to_string(), Value::Number(height));
    }

    Some(Value::Object(normalized))
}

fn non_empty_string<'a>(map: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    map.get(key).and_then(Value::as_str).filter(|value| !value.is_empty())
}

fn to_number(value: Option<&Value>) -> Option<Number> {
    match value? {
        Value::Number(number) => Some(number.clone()),
        Value::String(value) => value
            .chars()
            .all(|ch| ch.is_ascii_digit())
            .then_some(value)
            .filter(|value| !value.is_empty())
            .and_then(|value| value.parse::<u64>().ok())
            .map(Number::from),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::normalize_vitepress_frontmatter;

    #[test]
    fn normalizes_vitepress_home_frontmatter() {
        let frontmatter = normalize_vitepress_frontmatter(json!({
            "layout": "home",
            "hero": {
                "name": "Docs",
                "image": {
                    "light": "/logo-light.svg",
                    "dark": "/logo-dark.svg",
                    "width": "120",
                    "height": 80
                }
            }
        }));

        assert_eq!(
            frontmatter,
            json!({
                "layout": "entry",
                "hero": {
                    "name": "Docs",
                    "image": {
                        "src": "/logo-light.svg",
                        "lightSrc": "/logo-light.svg",
                        "darkSrc": "/logo-dark.svg",
                        "width": 120,
                        "height": 80
                    }
                }
            })
        );
    }

    #[test]
    fn preserves_ox_content_hero_image_theme_sources() {
        let frontmatter = normalize_vitepress_frontmatter(json!({
            "layout": "entry",
            "hero": {
                "name": "Ox Content",
                "image": {
                    "src": "oxcontent-dark.svg",
                    "lightSrc": "oxcontent-dark.svg",
                    "darkSrc": "oxcontent-light.svg",
                    "alt": "Ox Content wordmark"
                }
            }
        }));

        assert_eq!(
            frontmatter,
            json!({
                "layout": "entry",
                "hero": {
                    "name": "Ox Content",
                    "image": {
                        "src": "oxcontent-dark.svg",
                        "lightSrc": "oxcontent-dark.svg",
                        "darkSrc": "oxcontent-light.svg",
                        "alt": "Ox Content wordmark"
                    }
                }
            })
        );
    }

    #[test]
    fn preserves_hero_when_image_has_no_source() {
        let frontmatter = normalize_vitepress_frontmatter(json!({
            "layout": "home",
            "hero": {
                "name": "Docs",
                "image": {
                    "alt": "Logo"
                }
            }
        }));

        assert_eq!(
            frontmatter,
            json!({
                "layout": "entry",
                "hero": {
                    "name": "Docs",
                    "image": {
                        "alt": "Logo"
                    }
                }
            })
        );
    }
}
