//! Slide parsing and HTML shell generation for Ox Content.

mod models;
mod notes;
mod parser;
mod render;
mod theme;

pub use models::{
    DeckPrintRenderData, ExtractedSlideComments, ParsedSlideDeck, PrintSlideRenderData,
    SlideRenderData, SlideSource, SlideTheme,
};
pub use notes::extract_slide_comments;
pub use parser::parse_markdown_slide_deck;
pub use render::{generate_deck_print_html, generate_presenter_html, generate_slide_html};
