use super::{SharedCss, SkinManifest, read_skin_css};
use crate::Result;
use crate::fs::Workspace;
use regex::Regex;

fn authored_skin_sources() -> Result<Vec<(String, String)>> {
    let workspace = Workspace::discover()?;
    let manifest: SkinManifest =
        workspace.read_json(workspace.theme_skins_dir.join("skins.json"))?;
    let shared = SharedCss::read(&workspace)?;
    let mut sources = vec![
        ("shared/motion.css".to_string(), shared.motion),
        ("shared/hero.css".to_string(), shared.hero),
        ("shared/details.css".to_string(), shared.details),
        ("shared/guards.css".to_string(), shared.guards),
    ];

    for skin in &manifest.skins {
        sources.push((
            format!("skins/{}.css", skin.id),
            read_skin_css(&workspace.theme_skins_dir, &skin.id)?,
        ));
    }

    Ok(sources)
}

fn css_blocks(css: &str) -> Result<Vec<(&str, &str)>> {
    let block = Regex::new(r"(?s)([^{}]+)\{([^{}]*)\}")?;
    Ok(block
        .captures_iter(css)
        .map(|capture| {
            let selector = capture.get(1).map_or("", |value| value.as_str()).trim();
            let body = capture.get(2).map_or("", |value| value.as_str()).trim();
            (selector, body)
        })
        .collect())
}

#[test]
fn generated_skin_headers_do_not_use_backdrop_blur() -> Result<()> {
    for (name, css) in authored_skin_sources()? {
        for (selector, body) in css_blocks(&css)? {
            assert!(
                !(selector.contains(".header") && body.contains("backdrop-filter")),
                "{name} applies backdrop blur to header in selector {selector}"
            );
        }
    }

    Ok(())
}

#[test]
fn generated_skin_hover_states_do_not_toggle_italic() -> Result<()> {
    for (name, css) in authored_skin_sources()? {
        for (selector, body) in css_blocks(&css)? {
            assert!(
                !(selector.contains(":hover") && body.contains("font-style: italic")),
                "{name} toggles italic on hover in selector {selector}"
            );
        }
    }

    Ok(())
}

#[test]
fn paper_and_receipt_code_blocks_stay_flat_and_padded() -> Result<()> {
    let workspace = Workspace::discover()?;
    let paper = read_skin_css(&workspace.theme_skins_dir, "paper")?;
    for (selector, body) in css_blocks(&paper)? {
        if selector.contains(".content pre") {
            assert!(
                !body.contains("box-shadow")
                    && !body.contains("linear-gradient(")
                    && !body.contains("radial-gradient("),
                "Paper code block must stay flat in selector {selector}: {body}"
            );
        }
    }

    let receipt = read_skin_css(&workspace.theme_skins_dir, "receipt")?;
    for (selector, body) in css_blocks(&receipt)? {
        if selector.contains(".content pre") {
            assert!(
                !body.contains("padding-inline: 0"),
                "Receipt code block must keep readable inline padding in selector {selector}: {body}"
            );
        }
    }

    Ok(())
}
