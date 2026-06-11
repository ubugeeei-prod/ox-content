use crate::error::I18nError;
use crate::mf2::ast::{
    ComplexBody, ComplexMessage, Declaration, Expression, Message, Operand, Pattern, PatternPart,
    Variant,
};
use rustc_hash::FxHashSet;

/// Performs semantic validation on a parsed MF2 message.
pub fn validate(message: &Message) -> Vec<I18nError> {
    let mut errors = Vec::new();
    match message {
        Message::Simple(pattern) => validate_pattern(pattern, &FxHashSet::default(), &mut errors),
        Message::Complex(complex) => validate_complex(complex, &mut errors),
    }
    errors
}

fn validate_complex(complex: &ComplexMessage, errors: &mut Vec<I18nError>) {
    let mut declared_vars: FxHashSet<String> = FxHashSet::default();

    // Collect declared variables
    for decl in &complex.declarations {
        match decl {
            Declaration::Input(input) => {
                if !declared_vars.insert(input.variable.clone()) {
                    errors.push(I18nError::Mf2Validation {
                        message: format!(
                            "duplicate declaration of variable '${}''",
                            input.variable
                        ),
                    });
                }
            }
            Declaration::Local(local) => {
                // Validate the RHS expression first
                validate_expression(&local.expression, &declared_vars, errors);
                if !declared_vars.insert(local.variable.clone()) {
                    errors.push(I18nError::Mf2Validation {
                        message: format!(
                            "duplicate declaration of variable '${}''",
                            local.variable
                        ),
                    });
                }
            }
        }
    }

    match &complex.body {
        ComplexBody::Matcher(matcher) => {
            // All selectors must be declared
            for selector in &matcher.selectors {
                if !declared_vars.contains(selector) {
                    errors.push(I18nError::Mf2Validation {
                        message: format!(
                            "selector '${selector}' is not declared; add .input {{${selector} :...}}"
                        ),
                    });
                }
            }

            // Each variant must have matching key count
            let expected_keys = matcher.selectors.len();
            for variant in &matcher.variants {
                if variant.keys.len() != expected_keys {
                    errors.push(I18nError::Mf2Validation {
                        message: format!(
                            "variant has {} keys but {} selectors declared",
                            variant.keys.len(),
                            expected_keys
                        ),
                    });
                }
                validate_pattern(&variant.pattern, &declared_vars, errors);
            }

            // Must have a catch-all variant
            validate_catch_all(&matcher.variants, expected_keys, errors);
        }
        ComplexBody::QuotedPattern(pattern) => {
            validate_pattern(pattern, &declared_vars, errors);
        }
    }
}

fn validate_pattern(
    pattern: &Pattern,
    declared_vars: &FxHashSet<String>,
    errors: &mut Vec<I18nError>,
) {
    for part in &pattern.parts {
        if let PatternPart::Expression(expr) = part {
            validate_expression(expr, declared_vars, errors);
        }
    }
}

fn validate_expression(
    expr: &Expression,
    _declared_vars: &FxHashSet<String>,
    errors: &mut Vec<I18nError>,
) {
    // Validate that annotation options are not duplicated
    if let Some(ann) = &expr.annotation {
        let mut seen_opts: FxHashSet<String> = FxHashSet::default();
        for opt in &ann.options {
            if !seen_opts.insert(opt.name.clone()) {
                errors.push(I18nError::Mf2Validation {
                    message: format!("duplicate option '{}' in :{}", opt.name, ann.function),
                });
            }
        }
    }

    // Validate operand exists for expression without annotation
    if expr.operand.is_none() && expr.annotation.is_none() {
        errors.push(I18nError::Mf2Validation {
            message: "expression must have an operand or annotation".to_string(),
        });
    }
}

fn validate_catch_all(variants: &[Variant], selector_count: usize, errors: &mut Vec<I18nError>) {
    let has_catch_all = variants.iter().any(|v| {
        v.keys.len() == selector_count
            && v.keys.iter().all(|k| matches!(k, crate::mf2::ast::VariantKey::Wildcard))
    });

    if !has_catch_all {
        errors.push(I18nError::Mf2Validation {
            message: "matcher must include a catch-all variant with all wildcard (*) keys"
                .to_string(),
        });
    }
}

/// Extracts all variable names referenced in a message.
#[must_use]
pub fn extract_variables(message: &Message) -> FxHashSet<String> {
    let mut vars = FxHashSet::default();
    match message {
        Message::Simple(pattern) => extract_pattern_vars(pattern, &mut vars),
        Message::Complex(complex) => {
            for decl in &complex.declarations {
                match decl {
                    Declaration::Input(input) => {
                        vars.insert(input.variable.clone());
                    }
                    Declaration::Local(local) => {
                        vars.insert(local.variable.clone());
                        extract_expression_vars(&local.expression, &mut vars);
                    }
                }
            }
            match &complex.body {
                ComplexBody::Matcher(matcher) => {
                    for variant in &matcher.variants {
                        extract_pattern_vars(&variant.pattern, &mut vars);
                    }
                }
                ComplexBody::QuotedPattern(pattern) => {
                    extract_pattern_vars(pattern, &mut vars);
                }
            }
        }
    }
    vars
}

fn extract_pattern_vars(pattern: &Pattern, vars: &mut FxHashSet<String>) {
    for part in &pattern.parts {
        if let PatternPart::Expression(expr) = part {
            extract_expression_vars(expr, vars);
        }
    }
}

fn extract_expression_vars(expr: &Expression, vars: &mut FxHashSet<String>) {
    if let Some(Operand::Variable(name)) = &expr.operand {
        vars.insert(name.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mf2;

    #[test]
    fn valid_simple_message() {
        let msg = mf2::parse("Hello {$name}").unwrap();
        let errors = validate(&msg);
        assert!(errors.is_empty());
    }

    #[test]
    fn valid_complex_message() {
        let source = ".input {$count :number}\n.match $count\none {{one}}\n* {{other}}";
        let msg = mf2::parse(source).unwrap();
        let errors = validate(&msg);
        assert!(errors.is_empty());
    }

    #[test]
    fn missing_catch_all() {
        let source = ".input {$count :number}\n.match $count\none {{one}}";
        let msg = mf2::parse(source).unwrap();
        let errors = validate(&msg);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.to_string().contains("catch-all")));
    }

    #[test]
    fn duplicate_option() {
        let msg = mf2::parse("{$x :number style=decimal style=percent}").unwrap();
        let errors = validate(&msg);
        assert!(errors.iter().any(|e| e.to_string().contains("duplicate option")));
    }

    #[test]
    fn extract_variables_simple() {
        let msg = mf2::parse("Hello {$name}, you have {$count} items").unwrap();
        let vars = extract_variables(&msg);
        assert!(vars.contains("name"));
        assert!(vars.contains("count"));
    }

    #[test]
    fn extract_variables_complex() {
        let source = ".input {$count :number}\n.match $count\none {{You have {$count} item.}}\n* {{You have {$count} items.}}";
        let msg = mf2::parse(source).unwrap();
        let vars = extract_variables(&msg);
        assert!(vars.contains("count"));
    }
}
