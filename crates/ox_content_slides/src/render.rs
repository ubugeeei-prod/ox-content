use askama::Template;

use crate::{
    theme::{merge_theme, ResolvedSlideTheme},
    DeckPrintRenderData, PrintSlideRenderData, SlideRenderData, SlideTheme,
};

#[derive(Template)]
#[template(path = "slide.html")]
struct SlideTemplate<'a> {
    page_title: &'a str,
    description: Option<&'a str>,
    deck_title: &'a str,
    slide_title: &'a str,
    slide_number: u32,
    slide_count: u32,
    home_href: &'a str,
    slide_content_html: &'a str,
    has_previous: bool,
    previous_href: &'a str,
    has_next: bool,
    next_href: &'a str,
    has_presenter: bool,
    presenter_href: &'a str,
    previous_href_js: String,
    next_href_js: String,
    presenter_href_js: String,
    theme: &'a ResolvedSlideTheme<'a>,
}

#[derive(Template)]
#[template(path = "print.html")]
struct PrintTemplate<'a> {
    page_title: &'a str,
    description: Option<&'a str>,
    deck_title: &'a str,
    page_width: &'a str,
    page_height: &'a str,
    slides: &'a [PrintSlideRenderData],
    theme: &'a ResolvedSlideTheme<'a>,
}

#[derive(Template)]
#[template(path = "presenter.html")]
struct PresenterTemplate<'a> {
    page_title: &'a str,
    deck_title: &'a str,
    slide_title: &'a str,
    slide_number: u32,
    slide_count: u32,
    slide_href: &'a str,
    has_previous: bool,
    previous_href: &'a str,
    has_next: bool,
    next_href: &'a str,
    has_notes: bool,
    notes_html: &'a str,
    has_next_preview: bool,
    next_slide_href: &'a str,
    storage_key_js: String,
    previous_href_js: String,
    next_href_js: String,
    theme: &'a ResolvedSlideTheme<'a>,
}

/// Generates the standalone HTML shell for a slide page.
pub fn generate_slide_html(data: &SlideRenderData, theme: Option<&SlideTheme>) -> String {
    let theme = merge_theme(theme);
    let (has_previous, previous_href) = option_href_parts(data.previous_href.as_ref());
    let (has_next, next_href) = option_href_parts(data.next_href.as_ref());
    let (has_presenter, presenter_href) = option_href_parts(data.presenter_href.as_ref());

    SlideTemplate {
        page_title: &data.slide_title,
        description: data.slide_description.as_deref(),
        deck_title: &data.deck_title,
        slide_title: &data.slide_title,
        slide_number: data.slide_number,
        slide_count: data.slide_count,
        home_href: &data.home_href,
        slide_content_html: &data.slide_content_html,
        has_previous,
        previous_href,
        has_next,
        next_href,
        has_presenter,
        presenter_href,
        previous_href_js: option_js_string(data.previous_href.as_deref()),
        next_href_js: option_js_string(data.next_href.as_deref()),
        presenter_href_js: option_js_string(data.presenter_href.as_deref()),
        theme: &theme,
    }
    .render()
    .expect("slide template should render")
}

/// Generates the presenter-mode HTML shell for a slide page.
pub fn generate_presenter_html(data: &SlideRenderData, theme: Option<&SlideTheme>) -> String {
    let theme = merge_theme(theme);
    let (has_previous, previous_href) = option_href_parts(data.previous_href.as_ref());
    let (has_next, next_href) = option_href_parts(data.next_href.as_ref());
    let (has_next_preview, next_slide_href) = option_href_parts(data.next_slide_href.as_ref());
    let page_title = [data.slide_title.as_str(), " Presenter"].concat();
    let storage_key = ["ox-content:slides:timer:", data.deck_title.as_str()].concat();

    PresenterTemplate {
        page_title: &page_title,
        deck_title: &data.deck_title,
        slide_title: &data.slide_title,
        slide_number: data.slide_number,
        slide_count: data.slide_count,
        slide_href: &data.slide_href,
        has_previous,
        previous_href,
        has_next,
        next_href,
        has_notes: data.slide_notes_html.is_some(),
        notes_html: data.slide_notes_html.as_deref().unwrap_or(""),
        has_next_preview,
        next_slide_href,
        storage_key_js: js_string(&storage_key),
        previous_href_js: option_js_string(data.previous_href.as_deref()),
        next_href_js: option_js_string(data.next_href.as_deref()),
        theme: &theme,
    }
    .render()
    .expect("presenter template should render")
}

/// Generates the print-friendly HTML shell used for deck PDF export.
pub fn generate_deck_print_html(data: &DeckPrintRenderData, theme: Option<&SlideTheme>) -> String {
    let theme = merge_theme(theme);

    PrintTemplate {
        page_title: &data.deck_title,
        description: data.deck_description.as_deref(),
        deck_title: &data.deck_title,
        page_width: &data.page_width,
        page_height: &data.page_height,
        slides: &data.slides,
        theme: &theme,
    }
    .render()
    .expect("deck print template should render")
}

fn js_string(input: &str) -> String {
    serde_json::to_string(input).unwrap_or_else(|_| "\"\"".to_string())
}

fn option_js_string(input: Option<&str>) -> String {
    input.map_or_else(|| "null".to_string(), js_string)
}

fn option_href_parts(value: Option<&String>) -> (bool, &str) {
    value.map_or((false, ""), |href| (true, href.as_str()))
}

#[cfg(test)]
mod tests {
    use crate::{DeckPrintRenderData, PrintSlideRenderData, SlideRenderData, SlideTheme};

    use super::{generate_deck_print_html, generate_slide_html};

    #[test]
    fn generates_deck_print_html() {
        let html = generate_deck_print_html(
            &DeckPrintRenderData {
                deck_title: "Deck Title".to_string(),
                deck_description: Some("Deck Description".to_string()),
                page_width: "13.333in".to_string(),
                page_height: "7.5in".to_string(),
                slides: vec![PrintSlideRenderData {
                    slide_title: "One".to_string(),
                    slide_content_html: "<h1>One</h1>".to_string(),
                    slide_number: 1,
                    slide_count: 1,
                }],
            },
            None,
        );

        assert!(html.contains("Deck Title"));
        assert!(html.contains("13.333in"));
        assert!(html.contains("<h1>One</h1>"));
    }

    #[test]
    fn omits_builtin_animation_css_when_opted_out() {
        let html = generate_slide_html(
            &SlideRenderData {
                deck_title: "Deck Title".to_string(),
                slide_title: "Slide Title".to_string(),
                slide_description: None,
                slide_content_html: "<h1>Hello</h1>".to_string(),
                slide_notes_html: None,
                slide_number: 1,
                slide_count: 1,
                home_href: "/slides/".to_string(),
                slide_href: "/slides/1/".to_string(),
                presenter_href: None,
                previous_href: None,
                next_href: None,
                next_slide_href: None,
            },
            Some(&SlideTheme { builtin_animations: Some(false), ..SlideTheme::default() }),
        );

        assert!(!html.contains("@keyframes ox-slide-enter"));
    }

    #[test]
    fn includes_default_surface_radius() {
        let html = generate_slide_html(
            &SlideRenderData {
                deck_title: "Deck Title".to_string(),
                slide_title: "Slide Title".to_string(),
                slide_description: None,
                slide_content_html: "<h1>Hello</h1>".to_string(),
                slide_notes_html: None,
                slide_number: 1,
                slide_count: 1,
                home_href: "/slides/".to_string(),
                slide_href: "/slides/1/".to_string(),
                presenter_href: None,
                previous_href: None,
                next_href: None,
                next_slide_href: None,
            },
            None,
        );

        assert!(html.contains("--ox-slide-radius: 4px;"));
    }

    #[test]
    fn includes_dark_mode_tokens() {
        let html = generate_slide_html(
            &SlideRenderData {
                deck_title: "Deck Title".to_string(),
                slide_title: "Slide Title".to_string(),
                slide_description: None,
                slide_content_html: "<pre><code>Hello</code></pre>".to_string(),
                slide_notes_html: None,
                slide_number: 1,
                slide_count: 1,
                home_href: "/slides/".to_string(),
                slide_href: "/slides/1/".to_string(),
                presenter_href: None,
                previous_href: None,
                next_href: None,
                next_slide_href: None,
            },
            None,
        );

        assert!(html.contains("@media (prefers-color-scheme: dark)"));
        assert!(html.contains("--ox-slide-dark-code-surface: #1a1a20;"));
    }
}
