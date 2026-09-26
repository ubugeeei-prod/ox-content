//! Flow-only declarations: `opaque type`, `declare export`,
//! `declare module.exports`, `import typeof`, and component syntax.

use super::{CODE, LITERAL, Rewriter};

impl Rewriter<'_> {
    pub(super) fn keyword(&mut self, start: usize, end: usize) {
        match &self.src[start..end] {
            b"opaque" => self.opaque_type(start, end),
            b"declare" => {
                self.declare_module_exports(end);
                if self.in_declare_module(start) {
                    // Already ambient: TypeScript rejects a nested `declare`.
                    self.blank(start, end);
                } else {
                    self.declare_export(start, end);
                }
            }
            b"import" => self.import_typeof(end),
            b"component" if self.types[start] => self.component_type(start, end),
            b"component" => self.component_declaration(start, end),
            // `interface { ... }` inline interface types → object types.
            b"interface"
                if self.types[start]
                    && self.next_sig(end).is_some_and(|next| self.src[next] == b'{') =>
            {
                self.blank(start, end);
            }
            _ => {}
        }
    }

    pub(super) fn in_declare_module(&self, index: usize) -> bool {
        let mut depth = 0usize;
        for brace in (0..index).rev() {
            if self.kind[brace] != CODE {
                continue;
            }
            match self.src[brace] {
                b'}' => depth += 1,
                b'{' if depth > 0 => depth -= 1,
                b'{' => {
                    let Some(mut name) = self.prev_sig(brace) else { return false };
                    while name > 0 && self.kind[name] == LITERAL && self.kind[name - 1] == LITERAL {
                        name -= 1;
                    }
                    let Some(module) = self.prev_sig(name) else { return false };
                    return self.word_before(module + 1) == Some(b"module")
                        && self.prev_sig(module + 1 - 6).is_some_and(|declare| {
                            self.word_before(declare + 1) == Some(b"declare")
                        });
                }
                _ => {}
            }
        }
        false
    }

    /// `declare module.exports: T` → `declare const  exports: T`.
    pub(super) fn declare_module_exports(&mut self, end: usize) {
        let Some(module) = self.next_sig(end) else { return };
        if self.src[module..].starts_with(b"module.exports")
            && self.next_sig(module + 14).is_some_and(|colon| self.src[colon] == b':')
        {
            self.out[module..module + 14].copy_from_slice(b"const  exports");
        }
    }

    /// `component(props) renders? T` types → `any`.
    pub(super) fn component_type(&mut self, start: usize, end: usize) {
        let Some(mut open) = self.next_sig(end) else { return };
        if self.src[open] == b'<' {
            let Some(close) = self.find_angle_close(open) else { return };
            let Some(next) = self.next_sig(close + 1) else { return };
            open = next;
        }
        if self.src[open] != b'(' {
            return;
        }
        let Some(close) = self.find_close(open) else { return };
        let end = self.renders_clause_end(close + 1).unwrap_or(close + 1);
        self.blank(start, end);
        self.out[start..start + 3].copy_from_slice(b"any");
    }

    /// `component Name(props) renders T { ... }` → `function  Name(props) { ... }`.
    pub(super) fn component_declaration(&mut self, start: usize, end: usize) {
        if !self.at_statement_start(start) {
            return;
        }
        let Some((_, name_end)) = self.next_word(end) else { return };
        let Some(mut open) = self.next_sig(name_end) else { return };
        if self.src[open] == b'<' {
            let Some(close) = self.find_angle_close(open) else { return };
            let Some(next) = self.next_sig(close + 1) else { return };
            open = next;
        }
        if self.src[open] != b'(' {
            return;
        }
        self.out[start..end].copy_from_slice(b"function ");
        if let Some(close) = self.find_close(open)
            && let Some(renders_end) = self.renders_clause_end(close + 1)
        {
            self.blank(close + 1, renders_end);
        }
    }

    /// End of a `renders T` / `renders? T` / `renders* T` clause at `from`.
    pub(super) fn renders_clause_end(&self, from: usize) -> Option<usize> {
        let (_, word_end) = self.next_word(from).filter(|&(s, e)| &self.src[s..e] == b"renders")?;
        let mut operand = self.next_sig(word_end)?;
        if matches!(self.src[operand], b'?' | b'*') {
            operand = self.next_sig(operand + 1)?;
        }
        self.type_operand_end(operand)
    }

    /// `opaque type T: Super = U` → `type T = U`.
    pub(super) fn opaque_type(&mut self, start: usize, end: usize) {
        let Some(type_keyword) = self.next_word(end).filter(|&(s, e)| &self.src[s..e] == b"type")
        else {
            return;
        };
        self.blank(start, end);
        let Some((_, name_end)) = self.next_word(type_keyword.1) else { return };
        let mut cursor = name_end;
        if let Some(angle) = self.next_sig(cursor)
            && self.src[angle] == b'<'
            && let Some(close) = self.find_angle_close(angle)
        {
            cursor = close + 1;
        }
        let Some(colon) = self.next_sig(cursor).filter(|&colon| self.src[colon] == b':') else {
            // `declare opaque type Id;` → `type Id = any;`
            if cursor == name_end
                && self.next_sig(cursor).is_some_and(|semi| matches!(self.src[semi], b';' | b'\n'))
            {
                let name = &self.src[type_keyword.1..name_end];
                let name = &name[name.iter().position(|b| !b.is_ascii_whitespace()).unwrap_or(0)..];
                let mut replacement = b"type ".to_vec();
                replacement.extend_from_slice(name);
                replacement.extend_from_slice(b" = any");
                if replacement.len() <= name_end - start {
                    self.blank(start, name_end);
                    self.out[start..start + replacement.len()].copy_from_slice(&replacement);
                }
            }
            return;
        };
        match self.depth_zero_byte(colon + 1, b"=;\n") {
            Some(equals) if self.src[equals] == b'=' => self.blank(colon, equals),
            _ => self.out[colon] = b'=',
        }
    }

    /// `declare export function f(): void` → `export declare function ...`.
    pub(super) fn declare_export(&mut self, start: usize, end: usize) {
        let Some((export_start, export_end)) =
            self.next_word(end).filter(|&(s, e)| &self.src[s..e] == b"export")
        else {
            return;
        };
        let follow = self.next_sig(export_end);
        let swappable = self.src[end..export_start].iter().all(u8::is_ascii_whitespace)
            && !follow.is_some_and(|follow| {
                matches!(self.src[follow], b'{' | b'*')
                    || self.src[follow..].starts_with(b"default")
            });
        if swappable {
            let gap = export_start - end;
            self.out[start..start + 6].copy_from_slice(b"export");
            self.out[start + 6..start + 6 + gap].copy_from_slice(&self.src[end..export_start]);
            self.out[start + 6 + gap..export_end].copy_from_slice(b"declare");
        } else {
            self.blank(start, end);
        }
    }

    /// `import typeof X from` / `import { typeof X }` → `type`.
    pub(super) fn import_typeof(&mut self, end: usize) {
        if self.next_sig(end).is_none_or(|next| matches!(self.src[next], b'(' | b'.')) {
            return;
        }
        let mut cursor = end;
        while let Some((word_start, word_end)) = self.next_word_in_statement(cursor) {
            match &self.src[word_start..word_end] {
                b"from" => return,
                b"typeof" => self.out[word_start..word_end].copy_from_slice(b"type  "),
                _ => {}
            }
            cursor = word_end;
        }
    }
}
