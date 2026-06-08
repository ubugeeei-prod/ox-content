use ox_content_ast::Document;

use super::HtmlRenderer;
use crate::html::autolink::FirstByteIndex;
use crate::html::toc::{collect_inline_toc_entries, scan_document_for_render, DocumentRenderScan};

impl HtmlRenderer {
    /// Renders a committed streaming fragment while preserving cross-fragment state.
    ///
    /// This is intentionally separate from [`Self::render`], so normal one-shot
    /// rendering keeps its exact setup cost and behavior. Incremental callers use
    /// this to preserve heading ID de-duplication across committed fragments.
    #[must_use]
    pub fn render_incremental_fragment(&mut self, document: &Document<'_>) -> String {
        self.render_fragment(document)
    }

    /// Renders an unstable streaming fragment without mutating committed state.
    ///
    /// The returned HTML is meant to be replaceable by the next streaming update.
    #[must_use]
    pub fn render_provisional_fragment(&mut self, document: &Document<'_>) -> String {
        let document_scan = scan_document_for_render(document);
        let heading_id_counts =
            (document_scan.heading_count != 0).then(|| self.heading_id_counts.clone());
        let html = self.render_fragment_with_scan(document, document_scan);
        if let Some(heading_id_counts) = heading_id_counts {
            self.heading_id_counts = heading_id_counts;
        }
        html
    }

    /// Clears renderer state that spans incremental fragments.
    pub fn reset_incremental_state(&mut self) {
        self.output.clear();
        self.heading_id_counts.clear();
        self.toc_entries.clear();
        self.document_has_toc_marker = false;
        self.heading_text_scratch.clear();
        self.heading_slug_scratch.clear();
        self.in_link = false;
        self.autolink_index = None;
    }

    fn render_fragment(&mut self, document: &Document<'_>) -> String {
        crate::profile_span!("renderer::render_fragment");
        let document_scan = scan_document_for_render(document);
        self.render_fragment_with_scan(document, document_scan)
    }

    fn render_fragment_with_scan(
        &mut self,
        document: &Document<'_>,
        document_scan: DocumentRenderScan,
    ) -> String {
        self.output.clear();
        self.toc_entries.clear();
        self.document_has_toc_marker = document_scan.has_toc_marker;
        if self.document_has_toc_marker {
            collect_inline_toc_entries(document, self.options.toc_max_depth, &mut self.toc_entries);
        }
        self.heading_id_counts.reserve(document_scan.heading_count);
        self.autolink_index =
            if self.options.autolink_urls && !self.options.autolink_patterns.is_empty() {
                Some(FirstByteIndex::from_patterns(&self.options.autolink_patterns))
            } else {
                None
            };
        self.in_link = false;
        let estimated_len = (document.span.len() as usize).saturating_mul(2);
        if self.output.capacity() < estimated_len {
            self.output.reserve(estimated_len - self.output.capacity());
        }
        self.render_document(document);
        std::mem::take(&mut self.output)
    }
}
