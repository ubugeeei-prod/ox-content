use super::super::escape_html_text;
use super::parse::TableData;

pub(super) fn emit_table(data: &TableData, title: Option<&str>, out: &mut String) {
    out.push_str("<div class=\"ox-data-table\">\n");
    out.push_str("<div class=\"ox-data-table__scroll\">\n");
    out.push_str("<table class=\"ox-data-table__table\">\n");
    if let Some(title) = title.map(str::trim).filter(|value| !value.is_empty()) {
        out.push_str("<caption class=\"ox-data-table__caption\">");
        escape_html_text(title, out);
        out.push_str("</caption>\n");
    }
    out.push_str("<thead class=\"ox-data-table__head\"><tr class=\"ox-data-table__row\">");
    for header in &data.headers {
        out.push_str("<th class=\"ox-data-table__header\" scope=\"col\">");
        escape_html_text(header, out);
        out.push_str("</th>");
    }
    out.push_str("</tr></thead>\n<tbody class=\"ox-data-table__body\">\n");
    for row in &data.rows {
        out.push_str("<tr class=\"ox-data-table__row\">");
        for cell in row {
            out.push_str("<td class=\"ox-data-table__cell\">");
            escape_html_text(cell, out);
            out.push_str("</td>");
        }
        out.push_str("</tr>\n");
    }
    out.push_str("</tbody>\n</table>\n</div>\n</div>\n");
}
