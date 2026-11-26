/// The element `</name>` closes at `end`, as `(open_start, open_end,
/// close_end, name)`. One that starts before `floor` may come back as `None`
/// instead: everything before `floor` has been written out already.
pub(super) fn find_previous_wrapped_element(
    html: &str,
    end: usize,
    floor: usize,
    finder: &mut TagFinder,
) -> Option<(usize, usize, usize, String)> {
    let close_start = if finder.remember {
        // The `</` has to be the last one before `end` with no `>` between
        // it and the `>` at `end - 1`, so none before the previous `>` can
        // be it.
        if end == 0 || html.as_bytes()[end - 1] != b'>' {
            return None;
        }
        let previous = finder.last_before(html, ">", end - 1).filter(|at| *at >= floor);
        let from = previous.map_or(floor, |at| at.saturating_sub(1).max(floor));
        from + memchr::memmem::rfind(&html.as_bytes()[from..end], b"</")?
    } else {
        html[..end].rfind("</")?
    };
    if close_start + 2 >= end {
        return None;
    }
    let close_name_end = html[close_start + 2..end].find('>')? + close_start + 2;
    if close_name_end + 1 != end {
        return None;
    }
    let tag_name = html[close_start + 2..close_name_end].trim().to_ascii_lowercase();
    if tag_name.is_empty() || !is_attr_target_tag(&tag_name) {
        return None;
    }
    let open_marker = format!("<{tag_name}");
    let open_start = finder.last_before(html, &open_marker, close_start)?;
    if finder.remember && open_start < floor {
        return None;
    }
    let open_end = finder.tag_end(html, open_start)?;
    Some((open_start, open_end - 1, close_name_end + 1, tag_name))
}

/// The void element whose tag ends at `end`, with the same `floor` as
/// [`find_previous_wrapped_element`].
pub(super) fn find_previous_void_element(
    html: &str,
    end: usize,
    floor: usize,
    finder: &mut TagFinder,
) -> Option<(usize, usize, String)> {
    if finder.remember && (end == 0 || html.as_bytes()[end - 1] != b'>') {
        return None;
    }
    let open_start = finder.last_before(html, "<", end)?;
    if finder.remember && open_start < floor {
        return None;
    }
    let open_end = finder.tag_end(html, open_start)?;
    if open_end != end {
        return None;
    }
    let name_start = open_start + 1;
    let mut name_end = name_start;
    let bytes = html.as_bytes();
    while name_end < bytes.len()
        && !bytes[name_end].is_ascii_whitespace()
        && bytes[name_end] != b'>'
        && bytes[name_end] != b'/'
    {
        name_end += 1;
    }
    let tag_name = html[name_start..name_end].to_ascii_lowercase();
    if matches!(tag_name.as_str(), "img" | "br" | "hr" | "input") {
        Some((open_start, open_end - 1, tag_name))
    } else {
        None
    }
}

pub(super) fn is_adjacent_inline_attr_target_tag(tag: &str) -> bool {
    matches!(tag, "a" | "img" | "code" | "span")
}

pub(super) fn is_attr_target_tag(tag: &str) -> bool {
    matches!(
        tag,
        "h1" | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "a"
            | "img"
            | "code"
            | "pre"
            | "p"
            | "div"
            | "span"
            | "blockquote"
            | "table"
            | "tr"
            | "th"
            | "td"
            | "ul"
            | "ol"
            | "li"
    )
}

/// The searches attribute blocks make into the HTML in front of them.
///
/// Each block looks back for the element it applies to, and forward for the
/// `>` of a closing tag after it. Searched afresh, a block that applies to
/// nothing read back to the start of the line or document — `a{.b}` repeated
/// cost O(n²), 256 KiB of it 1.5 s — so what one search reads is kept for
/// the next. Blocks are visited in document order, so the positions asked
/// about only grow.
pub(in crate::features) struct TagFinder {
    /// Off, every search reads the HTML afresh, as the pass used to.
    pub(super) remember: bool,
    seen: Vec<Occurrences>,
    next_close: Option<(usize, Option<usize>)>,
    tag_end: Option<(usize, Option<usize>)>,
}

/// Where `needle` occurs in the HTML before `upto`.
struct Occurrences {
    needle: String,
    upto: usize,
    starts: Vec<usize>,
}

impl TagFinder {
    pub(in crate::features) const fn new(remember: bool) -> Self {
        Self { remember, seen: Vec::new(), next_close: None, tag_end: None }
    }

    /// Where the last `needle` that ends by `end` starts.
    pub(super) fn last_before(&mut self, html: &str, needle: &str, end: usize) -> Option<usize> {
        if !self.remember {
            return html[..end].rfind(needle);
        }
        let index = self.seen.iter().position(|seen| seen.needle == needle).unwrap_or_else(|| {
            self.seen.push(Occurrences { needle: needle.to_string(), upto: 0, starts: Vec::new() });
            self.seen.len() - 1
        });
        let seen = &mut self.seen[index];
        if end > seen.upto {
            // Pick up where the last search stopped, including a match that
            // straddles that point.
            let from = seen.starts.last().map_or(0, |last| last + needle.len());
            let from = from.max(seen.upto.saturating_sub(needle.len() - 1));
            let finder = memchr::memmem::Finder::new(needle.as_bytes());
            seen.starts.extend(finder.find_iter(&html.as_bytes()[from..end]).map(|at| from + at));
            seen.upto = end;
        }
        let before = seen.starts.partition_point(|start| start + needle.len() <= end);
        before.checked_sub(1).map(|index| seen.starts[index])
    }

    /// The first `>` at or after `from`.
    pub(super) fn next_close(&mut self, html: &str, from: usize) -> Option<usize> {
        if self.remember
            && let Some((at, found)) = self.next_close
            && at <= from
            && found.is_none_or(|found| found >= from)
        {
            return found;
        }
        let found = memchr::memchr(b'>', &html.as_bytes()[from..]).map(|at| from + at);
        self.next_close = Some((from, found));
        found
    }

    /// [`scan_tag_end`], remembered for the last tag asked about: blocks
    /// that look back to the same tag would otherwise each walk it again.
    pub(super) fn tag_end(&mut self, html: &str, start: usize) -> Option<usize> {
        if self.remember
            && let Some((at, end)) = self.tag_end
            && at == start
        {
            return end;
        }
        let end = scan_tag_end(html, start);
        self.tag_end = Some((start, end));
        end
    }
}

fn scan_tag_end(html: &str, start: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut i = start;
    let mut quote = None;
    while i < bytes.len() {
        match quote {
            Some(q) if bytes[i] == q => quote = None,
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => quote = Some(bytes[i]),
            None if bytes[i] == b'>' => return Some(i + 1),
            None => {}
        }
        i += 1;
    }
    None
}
