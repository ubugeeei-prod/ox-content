use ox_content_ast::Document;

mod blocks;
mod format;
mod inlines;
mod serializer;
#[cfg(test)]
mod tests;

pub use format::{MDAST_SECTION_CONTENT, MDAST_SECTION_FRONTMATTER, MDAST_SECTION_SOURCE_ORIGIN};

use serializer::MdastRawSerializer;

pub fn to_mdast_raw(document: &Document<'_>) -> crate::transfer::Result<Vec<u8>> {
    to_mdast_raw_with_sections(document, Vec::new())
}

pub fn to_mdast_raw_with_sections(
    document: &Document<'_>,
    extra_sections: Vec<(u32, Vec<u8>)>,
) -> crate::transfer::Result<Vec<u8>> {
    let mut serializer = MdastRawSerializer::default();
    let root_index = serializer.write_document(document)?;
    serializer.finish(root_index, extra_sections)
}
