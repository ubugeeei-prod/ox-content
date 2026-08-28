pub(super) fn decode_basic_entities(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let ent = &rest[amp..];
        if let Some((ch, skip)) = decode_one_entity(ent) {
            out.push(ch);
            rest = &ent[skip..];
        } else {
            out.push('&');
            rest = &ent[1..];
        }
    }
    out.push_str(rest);
    out
}

fn decode_one_entity(ent: &str) -> Option<(char, usize)> {
    const NAMED: &[(&str, char)] =
        &[("&amp;", '&'), ("&lt;", '<'), ("&gt;", '>'), ("&quot;", '"'), ("&apos;", '\'')];
    for (token, ch) in NAMED {
        if ent.starts_with(token) {
            return Some((*ch, token.len()));
        }
    }
    if let Some(rest) = ent.strip_prefix("&#x").or_else(|| ent.strip_prefix("&#X")) {
        let end = rest.find(';')?;
        let ch = char::from_u32(u32::from_str_radix(&rest[..end], 16).ok()?)?;
        return Some((ch, 3 + end + 1));
    }
    if let Some(rest) = ent.strip_prefix("&#") {
        let end = rest.find(';')?;
        let ch = char::from_u32(rest[..end].parse().ok()?)?;
        return Some((ch, 2 + end + 1));
    }
    None
}
