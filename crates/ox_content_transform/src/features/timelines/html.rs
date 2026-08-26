use super::super::{escape_html_attr, escape_html_text};
use super::parse::{TimelineDate, TimelineItem};

pub(super) fn emit_timeline(
    out: &mut String,
    caption: Option<&str>,
    ordered: bool,
    items: &[TimelineItem],
) {
    out.push_str("<section class=\"ox-timeline\" aria-label=\"Timeline\">\n");
    if let Some(caption) = caption.filter(|value| !value.trim().is_empty()) {
        out.push_str("<p class=\"ox-timeline__caption\">");
        escape_html_text(caption, out);
        out.push_str("</p>\n");
    }
    let list = if ordered { "ol" } else { "ul" };
    out.push('<');
    out.push_str(list);
    out.push_str(" class=\"ox-timeline__items\">\n");
    for item in items {
        emit_item(out, item);
    }
    out.push_str("</");
    out.push_str(list);
    out.push_str(">\n</section>\n");
}

fn emit_item(out: &mut String, item: &TimelineItem) {
    out.push_str("<li class=\"ox-timeline__item\"");
    if let Some(status) = item.status.as_deref() {
        out.push_str(" data-status=\"");
        escape_html_attr(status, out);
        out.push('"');
    }
    out.push_str(">\n<div class=\"ox-timeline__marker\" aria-hidden=\"true\"></div>\n");
    out.push_str("<div class=\"ox-timeline__content\">\n");
    emit_meta(out, item);
    if !item.title.is_empty() {
        emit_title(out, item);
    }
    if !item.body.trim().is_empty() {
        out.push('\n');
        out.push_str(item.body.trim_end());
        out.push('\n');
    }
    out.push_str("</div>\n</li>\n");
}

fn emit_meta(out: &mut String, item: &TimelineItem) {
    if item.date.is_none() && item.label.is_none() && item.status.is_none() {
        return;
    }
    out.push_str("<div class=\"ox-timeline__meta\">");
    if let Some(date) = &item.date {
        emit_date(out, date);
    }
    if let Some(label) = item.label.as_deref() {
        out.push_str("<span class=\"ox-timeline__label\">");
        escape_html_text(label, out);
        out.push_str("</span>");
    }
    if let Some(status) = item.status.as_deref() {
        out.push_str("<span class=\"ox-timeline__status\">");
        escape_html_text(status, out);
        out.push_str("</span>");
    }
    out.push_str("</div>\n");
}

fn emit_date(out: &mut String, date: &TimelineDate) {
    if let Some(datetime) = date.datetime.as_deref() {
        out.push_str("<time class=\"ox-timeline__date\" datetime=\"");
        escape_html_attr(datetime, out);
        out.push_str("\">");
        escape_html_text(&date.text, out);
        out.push_str("</time>");
    } else {
        out.push_str("<span class=\"ox-timeline__date ox-timeline__date--invalid\">");
        escape_html_text(&date.text, out);
        out.push_str("</span>");
    }
}

fn emit_title(out: &mut String, item: &TimelineItem) {
    out.push_str("<p class=\"ox-timeline__title\">");
    if let Some(href) = item.href.as_deref() {
        out.push_str("<a href=\"");
        escape_html_attr(href, out);
        out.push_str("\">");
        escape_html_text(&item.title, out);
        out.push_str("</a>");
    } else {
        escape_html_text(&item.title, out);
    }
    out.push_str("</p>\n");
}
