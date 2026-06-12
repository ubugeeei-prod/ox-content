use super::{tokenize, Token};

#[test]
fn simple_text() {
    let tokens = tokenize("Hello world").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::Text("Hello world".to_string()));
}

#[test]
fn simple_variable() {
    let tokens = tokenize("{$name}").unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].token, Token::OpenBrace);
    assert_eq!(tokens[1].token, Token::Variable("name".to_string()));
    assert_eq!(tokens[2].token, Token::CloseBrace);
}

#[test]
fn text_with_variable() {
    let tokens = tokenize("Hello {$name}!").unwrap();
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].token, Token::Text("Hello ".to_string()));
    assert_eq!(tokens[1].token, Token::OpenBrace);
    assert_eq!(tokens[2].token, Token::Variable("name".to_string()));
    assert_eq!(tokens[3].token, Token::CloseBrace);
    assert_eq!(tokens[4].token, Token::Text("!".to_string()));
}

#[test]
fn variable_with_function() {
    let tokens = tokenize("{$count :number}").unwrap();
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].token, Token::OpenBrace);
    assert_eq!(tokens[1].token, Token::Variable("count".to_string()));
    assert_eq!(tokens[2].token, Token::Function("number".to_string()));
    assert_eq!(tokens[3].token, Token::CloseBrace);
}

#[test]
fn function_with_option() {
    let tokens = tokenize("{$amount :number minimumFractionDigits=2}").unwrap();
    assert_eq!(tokens.len(), 7);
    assert_eq!(tokens[0].token, Token::OpenBrace);
    assert_eq!(tokens[1].token, Token::Variable("amount".to_string()));
    assert_eq!(tokens[2].token, Token::Function("number".to_string()));
    assert_eq!(tokens[3].token, Token::Name("minimumFractionDigits".to_string()));
    assert_eq!(tokens[4].token, Token::Equals);
    assert_eq!(tokens[5].token, Token::Number("2".to_string()));
    assert_eq!(tokens[6].token, Token::CloseBrace);
}

#[test]
fn dot_input() {
    let tokens = tokenize(".input {$count :number}").unwrap();
    assert_eq!(tokens[0].token, Token::DotInput);
}

#[test]
fn dot_match_with_variants() {
    let source = ".input {$count :number}\n.match $count\none {{Hello}}\n* {{default}}";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.iter().any(|t| t.token == Token::DotInput));
    assert!(tokens.iter().any(|t| t.token == Token::DotMatch));
    assert!(tokens.iter().any(|t| t.token == Token::Star));
    assert!(tokens.iter().any(|t| t.token == Token::DoubleOpenBrace));
}

#[test]
fn quoted_literal() {
    let tokens = tokenize("{|hello world| :string}").unwrap();
    assert_eq!(tokens[1].token, Token::QuotedLiteral("hello world".to_string()));
}

#[test]
fn text_with_punctuation() {
    let tokens = tokenize("You have {$count} items.").unwrap();
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].token, Token::Text("You have ".to_string()));
    assert_eq!(tokens[4].token, Token::Text(" items.".to_string()));
}
