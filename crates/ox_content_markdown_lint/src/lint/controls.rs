use super::{
    InternalMarkdownLintOptions, MarkdownLintDiagnostic, MarkdownLintSeverity, source::Source,
};
use ox_content_ast::Span;

#[derive(Default)]
pub(super) struct Controls {
    directives: Vec<Directive>,
}
struct Directive {
    line: usize,
    action: Action,
    rules: Vec<String>,
}
enum Action {
    Disable,
    Enable,
    NextLine,
}

impl Controls {
    pub fn collect(&mut self, source: &Source<'_>, span: Span) {
        let raw = &source.text[span.start as usize..span.end as usize];
        let mut offset = 0;
        while let Some(start) = raw[offset..].find("<!--") {
            let start = offset + start;
            let Some(end) = raw[start + 4..].find("-->") else {
                break;
            };
            let end = start + 4 + end;
            let content = raw[start + 4..end].trim();
            let (command, arguments) =
                content.split_once(char::is_whitespace).unwrap_or((content, ""));
            let action = match command {
                "oxlint-disable" => Action::Disable,
                "oxlint-enable" => Action::Enable,
                "oxlint-disable-next-line" => Action::NextLine,
                _ => {
                    offset = end + 3;
                    continue;
                }
            };
            self.directives.push(Directive {
                line: source.line_index(span.start as usize + end + 3) + 1,
                action,
                rules: arguments
                    .split([',', ' ', '\t', '\n', '\r'])
                    .filter(|v| !v.is_empty())
                    .map(str::to_string)
                    .collect(),
            });
            offset = end + 3;
        }
    }

    fn enabled(&self, rule: &str, line: usize) -> bool {
        let mut enabled = true;
        let mut next_line = false;
        for directive in &self.directives {
            if directive.line > line {
                break;
            }
            if !directive.rules.is_empty() && !directive.rules.iter().any(|v| v == rule) {
                continue;
            }
            match directive.action {
                Action::Disable => enabled = false,
                Action::Enable => enabled = true,
                Action::NextLine if directive.line + 1 == line => next_line = true,
                Action::NextLine => {}
            }
        }
        enabled && !next_line
    }

    pub fn apply(
        &self,
        diagnostics: &mut Vec<MarkdownLintDiagnostic>,
        options: &InternalMarkdownLintOptions,
    ) {
        diagnostics.retain_mut(|value| {
            if !self.enabled(&value.rule_id, value.line as usize) {
                return false;
            }
            if let Some(severity) = options.severities.get(&value.rule_id) {
                value.severity = match severity {
                    MarkdownLintSeverity::Off => return false,
                    MarkdownLintSeverity::Info => "info",
                    MarkdownLintSeverity::Warning => "warning",
                    MarkdownLintSeverity::Error => "error",
                }
                .into();
            }
            true
        });
    }

    pub fn spellcheck_enabled(&self, line: usize, options: &InternalMarkdownLintOptions) -> bool {
        self.enabled("spellcheck", line)
            && !matches!(options.severities.get("spellcheck"), Some(MarkdownLintSeverity::Off))
    }
}
