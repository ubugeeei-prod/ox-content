use napi_derive::napi;
use ox_content_slides::{DeckPrintRenderData, PrintSlideRenderData, SlideRenderData, SlideTheme};

/// Extracted slide comments result for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsExtractedSlideComments {
    pub content: String,
    pub notes: Vec<String>,
}

/// A parsed slide source for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsSlideSource {
    pub content: String,
    pub notes: Vec<String>,
}

/// Parsed Markdown slide deck for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsParsedSlideDeck {
    pub frontmatter: String,
    pub slides: Vec<JsSlideSource>,
}

/// Slide theme configuration for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsSlideTheme {
    pub aspect_ratio: Option<String>,
    pub max_width: Option<String>,
    pub max_height: Option<String>,
    pub padding: Option<String>,
    pub surface_radius: Option<String>,
    pub code_background: Option<String>,
    pub builtin_animations: Option<bool>,
    pub canvas_background: Option<String>,
    pub surface_background: Option<String>,
    pub surface_border: Option<String>,
    pub surface_shadow: Option<String>,
    pub presenter_sidebar_background: Option<String>,
    pub font_sans: Option<String>,
    pub font_mono: Option<String>,
    pub color_text: Option<String>,
    pub color_text_muted: Option<String>,
    pub color_primary: Option<String>,
    pub color_border: Option<String>,
    pub dark_canvas_background: Option<String>,
    pub dark_surface_background: Option<String>,
    pub dark_surface_border: Option<String>,
    pub dark_presenter_sidebar_background: Option<String>,
    pub dark_code_background: Option<String>,
    pub dark_color_text: Option<String>,
    pub dark_color_text_muted: Option<String>,
    pub dark_color_primary: Option<String>,
    pub dark_color_border: Option<String>,
}

/// Slide page render data for JavaScript.
#[napi(object)]
#[derive(Clone)]
pub struct JsSlideRenderData {
    pub deck_title: String,
    pub slide_title: String,
    pub slide_description: Option<String>,
    pub slide_content_html: String,
    pub slide_notes_html: Option<String>,
    pub slide_number: u32,
    pub slide_count: u32,
    pub home_href: String,
    pub slide_href: String,
    pub presenter_href: Option<String>,
    pub previous_href: Option<String>,
    pub next_href: Option<String>,
    pub next_slide_href: Option<String>,
}

/// Print shell render data for a single slide.
#[napi(object)]
#[derive(Clone)]
pub struct JsPrintSlideRenderData {
    pub slide_title: String,
    pub slide_content_html: String,
    pub slide_number: u32,
    pub slide_count: u32,
}

/// Deck-level print shell render data for JavaScript.
#[napi(object)]
#[derive(Clone)]
pub struct JsDeckPrintRenderData {
    pub deck_title: String,
    pub deck_description: Option<String>,
    pub page_width: String,
    pub page_height: String,
    pub slides: Vec<JsPrintSlideRenderData>,
}

fn convert_slide_theme(theme: Option<JsSlideTheme>) -> Option<SlideTheme> {
    theme.map(|t| SlideTheme {
        aspect_ratio: t.aspect_ratio,
        max_width: t.max_width,
        max_height: t.max_height,
        padding: t.padding,
        surface_radius: t.surface_radius,
        code_background: t.code_background,
        builtin_animations: t.builtin_animations,
        canvas_background: t.canvas_background,
        surface_background: t.surface_background,
        surface_border: t.surface_border,
        surface_shadow: t.surface_shadow,
        presenter_sidebar_background: t.presenter_sidebar_background,
        font_sans: t.font_sans,
        font_mono: t.font_mono,
        color_text: t.color_text,
        color_text_muted: t.color_text_muted,
        color_primary: t.color_primary,
        color_border: t.color_border,
        dark_canvas_background: t.dark_canvas_background,
        dark_surface_background: t.dark_surface_background,
        dark_surface_border: t.dark_surface_border,
        dark_presenter_sidebar_background: t.dark_presenter_sidebar_background,
        dark_code_background: t.dark_code_background,
        dark_color_text: t.dark_color_text,
        dark_color_text_muted: t.dark_color_text_muted,
        dark_color_primary: t.dark_color_primary,
        dark_color_border: t.dark_color_border,
    })
}

fn convert_slide_render_data(data: JsSlideRenderData) -> SlideRenderData {
    SlideRenderData {
        deck_title: data.deck_title,
        slide_title: data.slide_title,
        slide_description: data.slide_description,
        slide_content_html: data.slide_content_html,
        slide_notes_html: data.slide_notes_html,
        slide_number: data.slide_number,
        slide_count: data.slide_count,
        home_href: data.home_href,
        slide_href: data.slide_href,
        presenter_href: data.presenter_href,
        previous_href: data.previous_href,
        next_href: data.next_href,
        next_slide_href: data.next_slide_href,
    }
}

fn convert_deck_print_render_data(data: JsDeckPrintRenderData) -> DeckPrintRenderData {
    DeckPrintRenderData {
        deck_title: data.deck_title,
        deck_description: data.deck_description,
        page_width: data.page_width,
        page_height: data.page_height,
        slides: data
            .slides
            .into_iter()
            .map(|slide| PrintSlideRenderData {
                slide_title: slide.slide_title,
                slide_content_html: slide.slide_content_html,
                slide_number: slide.slide_number,
                slide_count: slide.slide_count,
            })
            .collect(),
    }
}

/// Extracts HTML comment-based speaker notes from a slide source.
#[napi]
pub fn extract_slide_comments(source: String) -> JsExtractedSlideComments {
    let extracted = ox_content_slides::extract_slide_comments(&source);
    JsExtractedSlideComments { content: extracted.content, notes: extracted.notes }
}

/// Parses a Markdown slide deck with optional frontmatter and `---` separators.
#[napi]
pub fn parse_markdown_slide_deck(source: String, separator: Option<String>) -> JsParsedSlideDeck {
    let parsed = ox_content_slides::parse_markdown_slide_deck(
        &source,
        separator.as_deref().unwrap_or("---"),
    );
    JsParsedSlideDeck {
        frontmatter: serde_json::to_string(&parsed.frontmatter)
            .unwrap_or_else(|_| "{}".to_string()),
        slides: parsed
            .slides
            .into_iter()
            .map(|slide| JsSlideSource { content: slide.content, notes: slide.notes })
            .collect(),
    }
}

/// Generates the standalone HTML shell for a slide page.
#[napi]
pub fn generate_slide_html(data: JsSlideRenderData, theme: Option<JsSlideTheme>) -> String {
    ox_content_slides::generate_slide_html(
        &convert_slide_render_data(data),
        convert_slide_theme(theme).as_ref(),
    )
}

/// Generates the presenter-mode HTML shell for a slide page.
#[napi]
pub fn generate_presenter_html(data: JsSlideRenderData, theme: Option<JsSlideTheme>) -> String {
    ox_content_slides::generate_presenter_html(
        &convert_slide_render_data(data),
        convert_slide_theme(theme).as_ref(),
    )
}

/// Generates a print-friendly HTML shell for deck-wide PDF export.
#[napi]
pub fn generate_deck_print_html(
    data: JsDeckPrintRenderData,
    theme: Option<JsSlideTheme>,
) -> String {
    ox_content_slides::generate_deck_print_html(
        &convert_deck_print_render_data(data),
        convert_slide_theme(theme).as_ref(),
    )
}
