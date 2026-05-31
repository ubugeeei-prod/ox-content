use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Slide source with extracted notes.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SlideSource {
    pub content: String,
    pub notes: Vec<String>,
}

/// HTML comments extracted from a slide source.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ExtractedSlideComments {
    pub content: String,
    pub notes: Vec<String>,
}

/// Parsed Markdown deck with frontmatter and slide bodies.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ParsedSlideDeck {
    pub frontmatter: JsonValue,
    pub slides: Vec<SlideSource>,
}

/// Theme values used by slide, presenter, and print shells.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SlideTheme {
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

/// Data used to render a standalone slide shell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideRenderData {
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

/// Data used to render a single slide into a deck print shell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintSlideRenderData {
    pub slide_title: String,
    pub slide_content_html: String,
    pub slide_number: u32,
    pub slide_count: u32,
}

/// Deck-level data used to render a print shell for PDF export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckPrintRenderData {
    pub deck_title: String,
    pub deck_description: Option<String>,
    pub page_width: String,
    pub page_height: String,
    pub slides: Vec<PrintSlideRenderData>,
}
