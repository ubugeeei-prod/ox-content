use super::MarkdownLintDiagnostic;

pub(super) fn apply(source: &str, diagnostics: &[MarkdownLintDiagnostic]) -> (String, u32) {
    let mut fixes = diagnostics.iter().filter_map(|v| v.fix.as_ref()).collect::<Vec<_>>();
    fixes.sort_unstable_by_key(|v| (v.start, std::cmp::Reverse(v.end)));
    fixes.dedup_by(|a, b| a == b);
    let mut output = String::with_capacity(source.len());
    let mut offset = 0;
    let mut applied = 0;
    let mut previous: Option<(u32, u32)> = None;
    for fix in fixes {
        let (start, end) = (fix.start as usize, fix.end as usize);
        if start < offset
            || end < start
            || end > source.len()
            || !source.is_char_boundary(start)
            || !source.is_char_boundary(end)
            || previous.is_some_and(|(a, b)| a == fix.start && b == fix.end)
        {
            continue;
        }
        output.push_str(&source[offset..start]);
        output.push_str(&fix.text);
        offset = end;
        previous = Some((fix.start, fix.end));
        applied += 1;
    }
    output.push_str(&source[offset..]);
    (output, applied)
}
