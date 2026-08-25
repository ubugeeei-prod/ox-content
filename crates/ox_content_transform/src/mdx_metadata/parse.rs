use super::cursor::{Cursor, is_ident_continue};
use super::{MdxImport, MdxImportSpecifier, MdxImportSpecifierKind, MdxMetadata, specifier};

pub(super) fn parse_mdxjs_esm(source: &str) -> MdxMetadata {
    let mut cursor = Cursor::new(source);
    let mut metadata = MdxMetadata::default();
    loop {
        cursor.skip_ws_and_comments();
        if cursor.is_eof() {
            break;
        }
        if cursor.eat_keyword("import") {
            if let Some(import) = parse_import(&mut cursor) {
                metadata.imports.push(import);
            } else {
                break;
            }
        } else if cursor.eat_keyword("export") {
            for name in parse_export(&mut cursor) {
                if !metadata.exports.contains(&name) {
                    metadata.exports.push(name);
                }
            }
        } else {
            break;
        }
    }
    metadata
}

fn parse_import(cursor: &mut Cursor<'_>) -> Option<MdxImport> {
    let _ = cursor.eat_keyword("type");
    cursor.skip_ws_and_comments();

    if let Some(source) = cursor.eat_string() {
        cursor.eat(';');
        return Some(MdxImport { source, specifiers: Vec::new() });
    }

    let mut specifiers = Vec::new();

    if cursor.eat('*') {
        if !cursor.eat_keyword("as") {
            cursor.skip_until_statement_end();
            return None;
        }
        let local = cursor.eat_ident()?;
        specifiers.push(specifier("*", local, MdxImportSpecifierKind::Namespace));
    } else if cursor.peek() == Some('{') {
        specifiers.extend(parse_named_import_list(cursor));
    } else if let Some(local) = cursor.eat_ident() {
        specifiers.push(specifier("default", local, MdxImportSpecifierKind::Default));
        cursor.skip_ws_and_comments();
        if cursor.eat(',') {
            cursor.skip_ws_and_comments();
            if cursor.eat('*') {
                if !cursor.eat_keyword("as") {
                    cursor.skip_until_statement_end();
                    return None;
                }
                let local = cursor.eat_ident()?;
                specifiers.push(specifier("*", local, MdxImportSpecifierKind::Namespace));
            } else {
                specifiers.extend(parse_named_import_list(cursor));
            }
        }
    } else {
        cursor.skip_until_statement_end();
        return None;
    }

    if !cursor.eat_keyword("from") {
        cursor.skip_until_statement_end();
        return None;
    }
    let source = cursor.eat_string()?;
    cursor.eat(';');
    Some(MdxImport { source, specifiers })
}

fn parse_named_import_list(cursor: &mut Cursor<'_>) -> Vec<MdxImportSpecifier> {
    let mut specifiers = Vec::new();
    if !cursor.eat('{') {
        return specifiers;
    }
    loop {
        cursor.skip_ws_and_comments();
        if cursor.eat('}') {
            break;
        }
        let _ = peek_type_modifier(cursor);
        let Some(imported) = cursor.eat_imported_name() else {
            break;
        };
        let local = if cursor.eat_keyword("as") {
            cursor.eat_ident().unwrap_or_else(|| imported.clone())
        } else {
            imported.clone()
        };
        specifiers.push(specifier(imported, local, MdxImportSpecifierKind::Named));
        cursor.eat(',');
        cursor.skip_ws_and_comments();
        if cursor.eat('}') {
            break;
        }
    }
    specifiers
}

/// `type` is a modifier when followed by an imported name, not by `as` or `}`.
fn peek_type_modifier(cursor: &mut Cursor<'_>) -> bool {
    cursor.skip_ws_and_comments();
    if !cursor.rest().starts_with("type") {
        return false;
    }
    let after = cursor.rest().get(4..).unwrap_or("");
    let next = after.chars().next();
    if next.is_some_and(is_ident_continue) {
        return false;
    }
    let mut lookahead = Cursor::new(after);
    lookahead.skip_ws_and_comments();
    if lookahead.eat_keyword("as") || lookahead.peek() == Some('}') || lookahead.peek() == Some(',')
    {
        return false;
    }
    if lookahead.eat_ident().is_none() && lookahead.eat_string().is_none() {
        return false;
    }
    cursor.consume_type_keyword();
    true
}

fn parse_export(cursor: &mut Cursor<'_>) -> Vec<String> {
    let _ = cursor.eat_keyword("type");
    let _ = cursor.eat_keyword("async");
    cursor.skip_ws_and_comments();

    if cursor.eat_keyword("default") {
        cursor.skip_until_statement_end();
        return vec!["default".to_string()];
    }

    if cursor.eat('*') {
        if cursor.eat_keyword("as") {
            let Some(name) = cursor.eat_ident() else {
                cursor.skip_until_statement_end();
                return Vec::new();
            };
            cursor.skip_until_statement_end();
            return vec![name];
        }
        cursor.skip_until_statement_end();
        return Vec::new();
    }

    if cursor.peek() == Some('{') {
        let names = parse_export_list(cursor);
        cursor.skip_until_statement_end();
        return names;
    }

    if cursor.eat_keyword("function") {
        cursor.eat('*');
        let names = cursor.eat_ident().into_iter().collect();
        cursor.skip_until_statement_end();
        return names;
    }

    if cursor.eat_keyword("class") || cursor.eat_keyword("interface") || cursor.eat_keyword("enum")
    {
        let names = cursor.eat_ident().into_iter().collect();
        cursor.skip_until_statement_end();
        return names;
    }

    if cursor.eat_keyword("const") || cursor.eat_keyword("let") || cursor.eat_keyword("var") {
        let names = parse_variable_export_names(cursor);
        cursor.skip_until_statement_end();
        return names;
    }

    if let Some(name) = cursor.eat_ident() {
        cursor.skip_until_statement_end();
        return vec![name];
    }

    cursor.skip_until_statement_end();
    Vec::new()
}

fn parse_export_list(cursor: &mut Cursor<'_>) -> Vec<String> {
    let mut names = Vec::new();
    if !cursor.eat('{') {
        return names;
    }
    loop {
        cursor.skip_ws_and_comments();
        if cursor.eat('}') {
            break;
        }
        let _ = peek_type_modifier(cursor);
        let Some(local) = cursor.eat_imported_name() else {
            break;
        };
        let exported = if cursor.eat_keyword("as") {
            cursor.eat_imported_name().unwrap_or(local)
        } else {
            local
        };
        names.push(exported);
        cursor.eat(',');
        cursor.skip_ws_and_comments();
        if cursor.eat('}') {
            break;
        }
    }
    names
}

fn parse_variable_export_names(cursor: &mut Cursor<'_>) -> Vec<String> {
    let mut names = Vec::new();
    loop {
        cursor.skip_ws_and_comments();
        if let Some(name) = cursor.eat_ident() {
            names.push(name);
        } else if cursor.peek() == Some('{') || cursor.peek() == Some('[') {
            let (open, close) = if cursor.peek() == Some('{') { ('{', '}') } else { ('[', ']') };
            cursor.skip_balanced(open, close);
        } else {
            break;
        }
        cursor.skip_ws_and_comments();
        if cursor.peek() == Some('=') {
            cursor.bump();
            skip_initializer(cursor);
        }
        cursor.skip_ws_and_comments();
        if !cursor.eat(',') {
            break;
        }
    }
    names
}

fn skip_initializer(cursor: &mut Cursor<'_>) {
    let mut brace = 0u32;
    let mut paren = 0u32;
    let mut bracket = 0u32;
    while !cursor.is_eof() {
        cursor.skip_ws_and_comments();
        match cursor.peek() {
            None => break,
            Some('\'') | Some('"') => {
                let _ = cursor.eat_string();
            }
            Some('{') => {
                brace += 1;
                cursor.bump();
            }
            Some('}') => {
                if brace == 0 {
                    break;
                }
                brace -= 1;
                cursor.bump();
            }
            Some('(') => {
                paren += 1;
                cursor.bump();
            }
            Some(')') => {
                paren = paren.saturating_sub(1);
                cursor.bump();
            }
            Some('[') => {
                bracket += 1;
                cursor.bump();
            }
            Some(']') => {
                bracket = bracket.saturating_sub(1);
                cursor.bump();
            }
            Some(',') | Some(';') if brace == 0 && paren == 0 && bracket == 0 => break,
            Some('\n') if brace == 0 && paren == 0 && bracket == 0 => break,
            _ => {
                cursor.bump();
            }
        }
    }
}
