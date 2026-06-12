use crate::mf2;
use crate::mf2::ast::{
    ComplexBody, Declaration, Message, Operand, OptionValue, PatternPart, VariantKey,
};

#[test]
fn simple_text() {
    let msg = mf2::parse("Hello world").unwrap();
    match msg {
        Message::Simple(pat) => {
            assert_eq!(pat.parts.len(), 1);
            assert!(matches!(&pat.parts[0], PatternPart::Text(t) if t == "Hello world"));
        }
        Message::Complex(_) => panic!("expected simple message"),
    }
}

#[test]
fn simple_variable() {
    let msg = mf2::parse("Hello {$name}").unwrap();
    match msg {
        Message::Simple(pat) => {
            assert_eq!(pat.parts.len(), 2);
            assert!(matches!(&pat.parts[0], PatternPart::Text(t) if t == "Hello "));
            match &pat.parts[1] {
                PatternPart::Expression(expr) => {
                    assert_eq!(expr.operand, Some(Operand::Variable("name".to_string())));
                }
                PatternPart::Text(_) => panic!("expected expression"),
            }
        }
        Message::Complex(_) => panic!("expected simple message"),
    }
}

#[test]
fn variable_with_function() {
    let msg = mf2::parse("{$amount :number minimumFractionDigits=2}").unwrap();
    match msg {
        Message::Simple(pat) => {
            assert_eq!(pat.parts.len(), 1);
            match &pat.parts[0] {
                PatternPart::Expression(expr) => {
                    assert_eq!(expr.operand, Some(Operand::Variable("amount".to_string())));
                    let ann = expr.annotation.as_ref().unwrap();
                    assert_eq!(ann.function, "number");
                    assert_eq!(ann.options.len(), 1);
                    assert_eq!(ann.options[0].name, "minimumFractionDigits");
                    assert_eq!(ann.options[0].value, OptionValue::Literal("2".to_string()));
                }
                PatternPart::Text(_) => panic!("expected expression"),
            }
        }
        Message::Complex(_) => panic!("expected simple message"),
    }
}

#[test]
fn complex_input_match() {
    let source = ".input {$count :number}\n.match $count\none {{You have {$count} notification.}}\n* {{You have {$count} notifications.}}";
    let msg = mf2::parse(source).unwrap();
    match msg {
        Message::Complex(cm) => {
            assert_eq!(cm.declarations.len(), 1);
            match &cm.declarations[0] {
                Declaration::Input(input) => {
                    assert_eq!(input.variable, "count");
                    assert_eq!(input.annotation.as_ref().unwrap().function, "number");
                }
                Declaration::Local(_) => panic!("expected input declaration"),
            }

            match &cm.body {
                ComplexBody::Matcher(matcher) => {
                    assert_eq!(matcher.selectors, vec!["count"]);
                    assert_eq!(matcher.variants.len(), 2);
                    assert_eq!(
                        matcher.variants[0].keys,
                        vec![VariantKey::Literal("one".to_string())]
                    );
                    assert_eq!(matcher.variants[1].keys, vec![VariantKey::Wildcard]);
                }
                ComplexBody::QuotedPattern(_) => panic!("expected matcher body"),
            }
        }
        Message::Simple(_) => panic!("expected complex message"),
    }
}

#[test]
fn local_declaration() {
    let source = ".local $greeting = {$name :string}\n.match $greeting\n* {{Hello {$greeting}}}";
    let msg = mf2::parse(source).unwrap();
    match msg {
        Message::Complex(cm) => {
            assert_eq!(cm.declarations.len(), 1);
            match &cm.declarations[0] {
                Declaration::Local(local) => {
                    assert_eq!(local.variable, "greeting");
                }
                Declaration::Input(_) => panic!("expected local declaration"),
            }
        }
        Message::Simple(_) => panic!("expected complex message"),
    }
}

#[test]
fn empty_message() {
    let msg = mf2::parse("").unwrap();
    match msg {
        Message::Simple(pat) => {
            assert!(pat.parts.is_empty());
        }
        Message::Complex(_) => panic!("expected simple message"),
    }
}

#[test]
fn text_with_punctuation() {
    let msg = mf2::parse("You have {$count} items.").unwrap();
    match msg {
        Message::Simple(pat) => {
            assert_eq!(pat.parts.len(), 3);
            assert!(matches!(&pat.parts[0], PatternPart::Text(t) if t == "You have "));
            assert!(matches!(&pat.parts[1], PatternPart::Expression(_)));
            assert!(matches!(&pat.parts[2], PatternPart::Text(t) if t == " items."));
        }
        Message::Complex(_) => panic!("expected simple message"),
    }
}
