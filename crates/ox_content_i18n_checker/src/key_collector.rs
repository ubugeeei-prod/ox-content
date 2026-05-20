use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_ast_visit::{walk, Visit};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::Path;

/// A collected translation key usage with source location.
#[derive(Debug, Clone)]
pub struct KeyUsage {
    pub key: String,
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub end_column: u32,
}

/// Extracts translation keys from TS/JS source files by finding `t('key')` calls.
pub struct KeyCollector {
    /// Function names to look for (default: `["t", "$t"]`).
    pub function_names: Vec<String>,
}

impl Default for KeyCollector {
    fn default() -> Self {
        Self { function_names: vec!["t".to_string(), "$t".to_string()] }
    }
}

impl KeyCollector {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a collector with custom function names.
    #[must_use]
    pub fn with_function_names(names: Vec<String>) -> Self {
        Self { function_names: names }
    }

    /// Collects translation keys from a source file.
    pub fn collect_file(&self, path: &Path) -> Result<Vec<KeyUsage>, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let file_path = path.to_string_lossy().to_string();
        let source_type = SourceType::from_path(path).unwrap_or_default();
        self.collect_source(&content, &file_path, source_type)
    }

    /// Collects translation keys from source code string.
    pub fn collect_source(
        &self,
        source: &str,
        file_path: &str,
        source_type: SourceType,
    ) -> Result<Vec<KeyUsage>, String> {
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source, source_type).parse();

        if !ret.errors.is_empty() {
            let msg = ret
                .errors
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!("parse error in {file_path}: {msg}"));
        }

        let mut visitor = KeyVisitor::new(source, file_path, &self.function_names);
        visitor.visit_program(&ret.program);

        Ok(visitor.usages)
    }
}

/// AST visitor that collects translation key usages.
struct KeyVisitor<'a> {
    source: &'a str,
    file_path: &'a str,
    function_names: &'a [String],
    usages: Vec<KeyUsage>,
}

impl<'a> KeyVisitor<'a> {
    fn new(source: &'a str, file_path: &'a str, function_names: &'a [String]) -> Self {
        Self { source, file_path, function_names, usages: Vec::new() }
    }

    fn line_col(&self, offset: u32) -> (u32, u32) {
        let bytes = self.source.as_bytes();
        let mut line = 1u32;
        let mut col = 1u32;
        for (i, &b) in bytes.iter().enumerate() {
            if i == offset as usize {
                break;
            }
            if b == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }
}

impl<'a> Visit<'a> for KeyVisitor<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        // Check if this is t('key') or $t('key')
        let callee_name = match &call.callee {
            Expression::Identifier(id) => Some(id.name.as_str()),
            // Handle member expressions like this.t('key') or i18n.t('key')
            Expression::StaticMemberExpression(member) => Some(member.property.name.as_str()),
            _ => None,
        };

        if let Some(name) = callee_name {
            if self.function_names.iter().any(|n| n == name) {
                // Extract the first string argument as the key
                if let Some(Argument::StringLiteral(lit)) = call.arguments.first() {
                    let (line, col) = self.line_col(call.span.start);
                    let (_, end_col) = self.line_col(call.span.end);
                    self.usages.push(KeyUsage {
                        key: lit.value.to_string(),
                        file_path: self.file_path.to_string(),
                        line,
                        column: col,
                        end_column: end_col,
                    });
                }
            }
        }

        // Continue visiting
        walk::walk_call_expression(self, call);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(source: &str) -> Vec<KeyUsage> {
        let collector = KeyCollector::new();
        collector.collect_source(source, "test.ts", SourceType::ts()).unwrap()
    }

    #[test]
    fn simple_t_call() {
        let usages = collect(r"const msg = t('common.greeting');");
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].key, "common.greeting");
    }

    #[test]
    fn dollar_t_call() {
        let usages = collect(r"const msg = $t('nav.home');");
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].key, "nav.home");
    }

    #[test]
    fn member_expression() {
        let usages = collect(r"const msg = i18n.t('common.farewell');");
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].key, "common.farewell");
    }

    #[test]
    fn multiple_calls() {
        let usages = collect(
            r"
const a = t('key1');
const b = t('key2');
const c = $t('key3');
",
        );
        assert_eq!(usages.len(), 3);
    }

    #[test]
    fn ignores_non_translation_calls() {
        let usages = collect(r"console.log('not a key'); foo('also not');");
        assert!(usages.is_empty());
    }

    #[test]
    fn ignores_non_string_arg() {
        let usages = collect(r"const msg = t(someVariable);");
        assert!(usages.is_empty());
    }

    #[test]
    fn line_column_tracking() {
        let usages = collect("const a = 1;\nconst b = t('key');");
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].line, 2);
    }
}
