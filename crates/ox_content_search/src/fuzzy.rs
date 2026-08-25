//! Bounded edit-distance helpers for optional search typo tolerance.

const MIN_FUZZY_CHARS: usize = 3;
const LONG_TOKEN_CHARS: usize = 6;
const ONE_EDIT_WEIGHT: f64 = 0.65;
const TWO_EDIT_WEIGHT: f64 = 0.40;

/// Returns a BM25 multiplier for a fuzzy term match.
///
/// The helper returns `None` for exact matches so callers can keep exact and
/// prefix scoring dominant. Distance work is bounded and exits as soon as a row
/// can no longer fit under the threshold.
pub fn fuzzy_match_weight(query: &str, candidate: &str) -> Option<f64> {
    let query_chars: Vec<char> = query.chars().collect();
    let candidate_chars: Vec<char> = candidate.chars().collect();
    let max_len = query_chars.len().max(candidate_chars.len());

    if max_len < MIN_FUZZY_CHARS {
        return None;
    }

    let max_distance = if max_len >= LONG_TOKEN_CHARS { 2 } else { 1 };
    if query_chars.len().abs_diff(candidate_chars.len()) > max_distance {
        return None;
    }

    let distance = bounded_edit_distance(&query_chars, &candidate_chars, max_distance)?;
    match distance {
        1 => Some(ONE_EDIT_WEIGHT),
        2 => Some(TWO_EDIT_WEIGHT),
        _ => None,
    }
}

fn bounded_edit_distance(left: &[char], right: &[char], max_distance: usize) -> Option<usize> {
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    let mut current = vec![0; right.len() + 1];

    for (row, left_char) in left.iter().enumerate() {
        current[0] = row + 1;
        let mut row_min = current[0];

        for (column, right_char) in right.iter().enumerate() {
            let substitution = usize::from(left_char != right_char);
            let value = (previous[column + 1] + 1)
                .min(current[column] + 1)
                .min(previous[column] + substitution);
            current[column + 1] = value;
            row_min = row_min.min(value);
        }

        if row_min > max_distance {
            return None;
        }
        std::mem::swap(&mut previous, &mut current);
    }

    let distance = previous[right.len()];
    (distance <= max_distance).then_some(distance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_matches_do_not_receive_fuzzy_weight() {
        assert_eq!(fuzzy_match_weight("install", "install"), None);
    }

    #[test]
    fn short_tokens_stay_precise() {
        assert_eq!(fuzzy_match_weight("go", "do"), None);
    }

    #[test]
    fn one_edit_typo_gets_stronger_weight() {
        assert_eq!(fuzzy_match_weight("instal", "install"), Some(ONE_EDIT_WEIGHT));
    }

    #[test]
    fn long_two_edit_typo_gets_weaker_weight() {
        assert_eq!(fuzzy_match_weight("serach", "search"), Some(TWO_EDIT_WEIGHT));
    }

    #[test]
    fn distant_terms_do_not_match() {
        assert_eq!(fuzzy_match_weight("search", "render"), None);
    }
}
