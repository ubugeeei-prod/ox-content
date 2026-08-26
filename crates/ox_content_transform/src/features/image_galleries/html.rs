use super::super::images::{self, ParsedImage, ResolvedImageOptions};
use super::GalleryItem;
use crate::features::attr_tokens::{ParsedAttrs, write_attrs_except};
use crate::features::{escape_html_attr, escape_html_text};

pub(super) fn emit_gallery(
    out: &mut String,
    caption: Option<&str>,
    items: &[GalleryItem<'_>],
    image_options: &ResolvedImageOptions,
) {
    out.push_str("<figure class=\"ox-image-gallery\"");
    if caption.is_none() {
        out.push_str(" role=\"group\" aria-label=\"Image gallery\"");
    }
    out.push_str(">\n");
    if let Some(caption) = caption.filter(|value| !value.trim().is_empty()) {
        out.push_str("<figcaption class=\"ox-image-gallery__caption\">");
        escape_html_text(caption, out);
        out.push_str("</figcaption>\n");
    }
    out.push_str("<ul class=\"ox-image-gallery__items\">\n");
    for item in items {
        out.push_str(
            "<li class=\"ox-image-gallery__item\"><figure class=\"ox-image-gallery__figure\">",
        );
        emit_img(out, &item.image, image_options);
        if let Some(caption) = item.image.caption {
            out.push_str("<figcaption class=\"ox-image-gallery__item-caption\">");
            let caption = images::unescape_markdown(caption);
            escape_html_text(&caption, out);
            out.push_str("</figcaption>");
        }
        out.push_str("</figure></li>\n");
    }
    out.push_str("</ul>\n</figure>\n");
}

fn emit_img(out: &mut String, image: &ParsedImage<'_>, options: &ResolvedImageOptions) {
    out.push_str("<img");
    if let Some(src) = image.src {
        out.push_str(" src=\"");
        escape_html_attr(src, out);
        out.push('"');
    }
    out.push_str(" alt=\"");
    escape_html_attr(image.alt, out);
    out.push('"');
    if let Some(attrs) = &image.attrs {
        write_image_attrs(out, attrs);
    }
    if options.lazy {
        out.push_str(" loading=\"lazy\"");
    }
    if let Some(width) = image.width.as_deref() {
        out.push_str(" width=\"");
        out.push_str(width);
        out.push('"');
    }
    if let Some(height) = image.height.as_deref() {
        out.push_str(" height=\"");
        out.push_str(height);
        out.push('"');
    }
    out.push('>');
}

fn write_image_attrs(out: &mut String, attrs: &ParsedAttrs) {
    write_attrs_except(out, attrs, &["src", "alt", "loading", "width", "height"]);
}
