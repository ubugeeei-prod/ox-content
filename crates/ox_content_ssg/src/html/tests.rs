fn snapshot_text(value: &str) -> String {
    let mut rendered = String::new();
    for segment in value.split_inclusive('\n') {
        let (line, has_newline) =
            segment.strip_suffix('\n').map_or((segment, false), |line| (line, true));
        let trimmed = line.trim_end_matches([' ', '\t']);
        rendered.push_str(trimmed);
        for ch in line[trimmed.len()..].chars() {
            match ch {
                ' ' => rendered.push_str("<sp>"),
                '\t' => rendered.push_str("<tab>"),
                _ => rendered.push(ch),
            }
        }
        if has_newline {
            rendered.push('\n');
        }
    }
    rendered
}

mod rendering;
mod theme;
