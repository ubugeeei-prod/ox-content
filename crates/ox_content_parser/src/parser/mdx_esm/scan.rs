//! Conservative ESM statement scan. Invalid input returns `None`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EsmScan<'a> {
    pub value: &'a str,
    pub end: usize,
    pub module_source: Option<&'a str>,
}

pub(super) fn scan_esm(source: &str, start: usize) -> Option<EsmScan<'_>> {
    let bytes = source.as_bytes();
    let cursor = start + 6;
    if match_word(bytes, start, b"import") {
        scan_import(source, start, cursor)
    } else if match_word(bytes, start, b"export") {
        scan_export(source, start, cursor)
    } else {
        None
    }
}

fn scan_import(source: &str, start: usize, mut cursor: usize) -> Option<EsmScan<'_>> {
    let bytes = source.as_bytes();
    skip_after_keyword(bytes, &mut cursor)?;
    if matches!(bytes.get(cursor), Some(b'"' | b'\'')) {
        let (module_source, next) = scan_string(source, cursor)?;
        return finish(source, start, next, Some(module_source));
    }
    cursor = scan_import_clause(source, cursor)?;
    expect_word(bytes, &mut cursor, b"from")?;
    skip_stmt_ws(bytes, &mut cursor)?;
    let (module_source, next) = scan_string(source, cursor)?;
    finish(source, start, next, Some(module_source))
}

fn scan_import_clause(source: &str, mut cursor: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    skip_stmt_ws(bytes, &mut cursor)?;
    match bytes.get(cursor)? {
        b'*' => scan_namespace_import(bytes, cursor),
        b'{' => scan_named_imports(source, cursor),
        _ => {
            cursor = scan_ident(bytes, cursor)?;
            let after_default = cursor;
            skip_stmt_ws(bytes, &mut cursor)?;
            if bytes.get(cursor) != Some(&b',') {
                return Some(after_default);
            }
            cursor += 1;
            skip_stmt_ws(bytes, &mut cursor)?;
            match bytes.get(cursor)? {
                b'*' => scan_namespace_import(bytes, cursor),
                b'{' => scan_named_imports(source, cursor),
                _ => None,
            }
        }
    }
}

fn scan_namespace_import(bytes: &[u8], mut cursor: usize) -> Option<usize> {
    if bytes.get(cursor)? != &b'*' {
        return None;
    }
    cursor += 1;
    expect_word(bytes, &mut cursor, b"as")?;
    skip_stmt_ws(bytes, &mut cursor)?;
    scan_ident(bytes, cursor)
}

fn scan_named_imports(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if bytes.get(start)? != &b'{' {
        return None;
    }
    let mut cursor = start + 1;
    skip_stmt_ws(bytes, &mut cursor)?;
    if bytes.get(cursor) == Some(&b'}') {
        return Some(cursor + 1);
    }
    loop {
        cursor = scan_import_specifier(source, cursor)?;
        skip_stmt_ws(bytes, &mut cursor)?;
        match bytes.get(cursor)? {
            b',' => {
                cursor += 1;
                skip_stmt_ws(bytes, &mut cursor)?;
                if bytes.get(cursor) == Some(&b'}') {
                    return Some(cursor + 1);
                }
            }
            b'}' => return Some(cursor + 1),
            _ => return None,
        }
    }
}

fn scan_import_specifier(source: &str, mut cursor: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    skip_stmt_ws(bytes, &mut cursor)?;
    if matches!(bytes.get(cursor), Some(b'"' | b'\'')) {
        cursor = skip_quoted(bytes, cursor)?;
        expect_word(bytes, &mut cursor, b"as")?;
        skip_stmt_ws(bytes, &mut cursor)?;
        return scan_ident(bytes, cursor);
    }
    cursor = scan_ident(bytes, cursor)?;
    let after_name = cursor;
    skip_stmt_ws(bytes, &mut cursor)?;
    if !match_word(bytes, cursor, b"as") {
        return Some(after_name);
    }
    cursor += 2;
    skip_stmt_ws(bytes, &mut cursor)?;
    scan_ident(bytes, cursor)
}

fn scan_export(source: &str, start: usize, mut cursor: usize) -> Option<EsmScan<'_>> {
    let bytes = source.as_bytes();
    skip_after_keyword(bytes, &mut cursor)?;
    if match_word(bytes, cursor, b"default") {
        cursor += 7;
        skip_stmt_ws(bytes, &mut cursor)?;
        cursor = scan_primary(bytes, cursor)?;
        return finish(source, start, cursor, None);
    }
    if match_word(bytes, cursor, b"const") || match_word(bytes, cursor, b"let") {
        cursor += if bytes[cursor] == b'c' { 5 } else { 3 };
        return scan_export_binding(source, start, cursor);
    }
    if match_word(bytes, cursor, b"var") {
        cursor += 3;
        return scan_export_binding(source, start, cursor);
    }
    None
}

fn scan_export_binding(source: &str, start: usize, mut cursor: usize) -> Option<EsmScan<'_>> {
    let bytes = source.as_bytes();
    skip_stmt_ws(bytes, &mut cursor)?;
    cursor = scan_ident(bytes, cursor)?;
    skip_stmt_ws(bytes, &mut cursor)?;
    if bytes.get(cursor)? != &b'=' {
        return None;
    }
    cursor += 1;
    skip_stmt_ws(bytes, &mut cursor)?;
    cursor = scan_primary(bytes, cursor)?;
    finish(source, start, cursor, None)
}

fn scan_primary(bytes: &[u8], cursor: usize) -> Option<usize> {
    match bytes.get(cursor)? {
        b'"' | b'\'' => skip_quoted(bytes, cursor),
        b'{' => skip_balanced(bytes, cursor, b'{', b'}'),
        b'[' => skip_balanced(bytes, cursor, b'[', b']'),
        b'(' => skip_balanced(bytes, cursor, b'(', b')'),
        b'0'..=b'9' => {
            let mut end = cursor + 1;
            while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b'.') {
                end += 1;
            }
            Some(end)
        }
        byte if is_ident_start(*byte) => scan_ident(bytes, cursor),
        _ => None,
    }
}

fn finish<'a>(
    source: &'a str,
    start: usize,
    mut cursor: usize,
    module_source: Option<&'a str>,
) -> Option<EsmScan<'a>> {
    let bytes = source.as_bytes();
    skip_line_ws(bytes, &mut cursor);
    if bytes.get(cursor) == Some(&b';') {
        cursor += 1;
        skip_line_ws(bytes, &mut cursor);
    }
    if bytes.get(cursor) == Some(&b'/') && bytes.get(cursor.saturating_add(1)) == Some(&b'/') {
        cursor = line_end(bytes, cursor);
    }
    if cursor < bytes.len() && bytes[cursor] != b'\n' {
        return None;
    }
    let mut value_end = cursor;
    while value_end > start && matches!(bytes[value_end - 1], b' ' | b'\t' | b'\r') {
        value_end -= 1;
    }
    let end = if cursor < bytes.len() { cursor + 1 } else { cursor };
    Some(EsmScan { value: &source[start..value_end], end, module_source })
}

fn scan_string(source: &str, start: usize) -> Option<(&str, usize)> {
    let end = skip_quoted(source.as_bytes(), start)?;
    Some((&source[start + 1..end - 1], end))
}

fn expect_word(bytes: &[u8], cursor: &mut usize, word: &[u8]) -> Option<()> {
    skip_stmt_ws(bytes, cursor)?;
    if !match_word(bytes, *cursor, word) {
        return None;
    }
    *cursor += word.len();
    Some(())
}

fn skip_after_keyword(bytes: &[u8], cursor: &mut usize) -> Option<()> {
    match bytes.get(*cursor) {
        Some(b' ' | b'\t' | b'\r' | b'\n' | b'/') => skip_stmt_ws(bytes, cursor),
        Some(b'{' | b'*' | b'"' | b'\'') => Some(()),
        _ => None,
    }
}

fn skip_stmt_ws(bytes: &[u8], cursor: &mut usize) -> Option<()> {
    loop {
        skip_line_ws(bytes, cursor);
        match bytes.get(*cursor) {
            Some(b'/') if bytes.get(cursor.saturating_add(1)) == Some(&b'/') => {
                *cursor = line_end(bytes, *cursor);
            }
            Some(b'/') if bytes.get(cursor.saturating_add(1)) == Some(&b'*') => {
                *cursor = skip_block_comment(bytes, *cursor)?;
            }
            Some(b'\n') => {
                let mut peek = *cursor + 1;
                skip_line_ws(bytes, &mut peek);
                if peek >= bytes.len() || bytes[peek] == b'\n' {
                    return None;
                }
                *cursor = peek;
            }
            _ => return Some(()),
        }
    }
}

fn skip_line_ws(bytes: &[u8], cursor: &mut usize) {
    while matches!(bytes.get(*cursor), Some(b' ' | b'\t' | b'\r')) {
        *cursor += 1;
    }
}

fn line_end(bytes: &[u8], mut cursor: usize) -> usize {
    while cursor < bytes.len() && bytes[cursor] != b'\n' {
        cursor += 1;
    }
    cursor
}

fn skip_block_comment(bytes: &[u8], start: usize) -> Option<usize> {
    let mut cursor = start + 2;
    while cursor + 1 < bytes.len() {
        if bytes[cursor] == b'*' && bytes[cursor + 1] == b'/' {
            return Some(cursor + 2);
        }
        cursor += 1;
    }
    None
}

fn skip_quoted(bytes: &[u8], start: usize) -> Option<usize> {
    let quote = *bytes.get(start)?;
    if !matches!(quote, b'"' | b'\'') {
        return None;
    }
    let mut cursor = start + 1;
    while cursor < bytes.len() && bytes[cursor] != b'\n' {
        if bytes[cursor] == b'\\' && cursor + 1 < bytes.len() {
            cursor += 2;
            continue;
        }
        if bytes[cursor] == quote {
            return Some(cursor + 1);
        }
        cursor += 1;
    }
    None
}

fn skip_balanced(bytes: &[u8], start: usize, open: u8, close: u8) -> Option<usize> {
    if bytes.get(start) != Some(&open) {
        return None;
    }
    let mut cursor = start + 1;
    let mut depth = 1u32;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'"' | b'\'' => cursor = skip_quoted(bytes, cursor)?,
            b'/' if bytes.get(cursor.saturating_add(1)) == Some(&b'/') => {
                cursor = line_end(bytes, cursor);
            }
            b'/' if bytes.get(cursor.saturating_add(1)) == Some(&b'*') => {
                cursor = skip_block_comment(bytes, cursor)?;
            }
            byte if byte == open => {
                depth = depth.saturating_add(1);
                cursor += 1;
            }
            byte if byte == close => {
                cursor += 1;
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(cursor);
                }
            }
            _ => cursor += 1,
        }
    }
    None
}

fn scan_ident(bytes: &[u8], start: usize) -> Option<usize> {
    if !bytes.get(start).is_some_and(|byte| is_ident_start(*byte)) {
        return None;
    }
    let mut end = start + 1;
    while end < bytes.len() && is_ident_continue(bytes[end]) {
        end += 1;
    }
    Some(end)
}

fn match_word(bytes: &[u8], at: usize, word: &[u8]) -> bool {
    let Some(rest) = bytes.get(at..) else {
        return false;
    };
    rest.starts_with(word) && !rest.get(word.len()).is_some_and(|byte| is_ident_continue(*byte))
}

fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$')
}

fn is_ident_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
}
