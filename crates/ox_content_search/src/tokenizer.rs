//! Text tokenization for search indexing.

use compact_str::CompactString;

/// Tokenizes text into searchable terms.
///
/// This tokenizer:
/// - Converts to lowercase
/// - Splits on whitespace and punctuation
/// - Filters out stopwords
/// - Handles CJK characters (Japanese, Chinese, Korean)
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = CompactString::default();

    for c in text.chars() {
        if is_cjk_char(c) {
            // CJK characters are treated as individual tokens
            if !current_token.is_empty() {
                let token = current_token.to_lowercase();
                if !is_stopword(&token) && token.len() >= 2 {
                    tokens.push(token.into_string());
                }
                current_token.clear();
            }
            tokens.push(String::from(c));
        } else if c.is_alphanumeric() || c == '_' {
            current_token.push(c);
        } else if !current_token.is_empty() {
            let token = current_token.to_lowercase();
            if !is_stopword(&token) && token.len() >= 2 {
                tokens.push(token.into_string());
            }
            current_token.clear();
        }
    }

    // Don't forget the last token
    if !current_token.is_empty() {
        let token = current_token.to_lowercase();
        if !is_stopword(&token) && token.len() >= 2 {
            tokens.push(token.into_string());
        }
    }

    tokens
}

/// Tokenizes text for query (less strict than indexing).
pub fn tokenize_query(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = CompactString::default();

    for c in text.chars() {
        if is_cjk_char(c) {
            if !current_token.is_empty() {
                let token = current_token.to_lowercase();
                tokens.push(token.into_string());
                current_token.clear();
            }
            tokens.push(String::from(c));
        } else if c.is_alphanumeric() || c == '_' {
            current_token.push(c);
        } else if !current_token.is_empty() {
            let token = current_token.to_lowercase();
            tokens.push(token.into_string());
            current_token.clear();
        }
    }

    if !current_token.is_empty() {
        let token = current_token.to_lowercase();
        tokens.push(token.into_string());
    }

    tokens
}

/// Checks if a character is a CJK character.
fn is_cjk_char(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |   // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' |   // CJK Extension A
        '\u{3040}'..='\u{309F}' |   // Hiragana
        '\u{30A0}'..='\u{30FF}' |   // Katakana
        '\u{AC00}'..='\u{D7AF}'     // Hangul Syllables
    )
}

/// Common English stopwords to filter out.
fn is_stopword(word: &str) -> bool {
    matches!(
        word,
        "a" | "an"
            | "and"
            | "are"
            | "as"
            | "at"
            | "be"
            | "by"
            | "for"
            | "from"
            | "has"
            | "he"
            | "in"
            | "is"
            | "it"
            | "its"
            | "of"
            | "on"
            | "or"
            | "that"
            | "the"
            | "to"
            | "was"
            | "were"
            | "will"
            | "with"
            | "this"
            | "but"
            | "they"
            | "have"
            | "had"
            | "what"
            | "when"
            | "where"
            | "who"
            | "which"
            | "why"
            | "how"
            | "all"
            | "each"
            | "every"
            | "both"
            | "few"
            | "more"
            | "most"
            | "other"
            | "some"
            | "such"
            | "no"
            | "not"
            | "only"
            | "own"
            | "same"
            | "so"
            | "than"
            | "too"
            | "very"
            | "can"
            | "just"
            | "should"
            | "now"
            | "if"
            | "you"
            | "your"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_english() {
        let tokens = tokenize("Hello, World! This is a test.");
        assert_eq!(tokens, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_tokenize_japanese() {
        let tokens = tokenize("これはテストです");
        assert_eq!(tokens, vec!["こ", "れ", "は", "テ", "ス", "ト", "で", "す"]);
    }

    #[test]
    fn test_tokenize_mixed() {
        let tokens = tokenize("Rustで検索エンジン");
        assert_eq!(tokens, vec!["rust", "で", "検", "索", "エ", "ン", "ジ", "ン"]);
    }

    #[test]
    fn test_tokenize_code() {
        let tokens = tokenize("function_name variable_name");
        assert_eq!(tokens, vec!["function_name", "variable_name"]);
    }
}
