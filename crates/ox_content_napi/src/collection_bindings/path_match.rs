use std::cmp::Ordering;
use std::path::Path;

pub(super) struct GlobPattern {
    segments: Vec<String>,
}

impl GlobPattern {
    fn new(pattern: &str) -> Self {
        let normalized = path_to_slash(pattern);
        let pattern = normalized.trim_start_matches("./").trim_start_matches('/');
        Self {
            segments: pattern
                .split('/')
                .filter(|segment| !segment.is_empty())
                .map(str::to_string)
                .collect(),
        }
    }

    pub(super) fn matches(&self, path: &str) -> bool {
        let segments: Vec<&str> = path.split('/').filter(|segment| !segment.is_empty()).collect();
        matches_glob_segments(&self.segments, &segments)
    }
}

pub(super) fn normalize_source_patterns(patterns: &[String]) -> Vec<GlobPattern> {
    if patterns.is_empty() {
        return vec![GlobPattern::new("**/*")];
    }
    patterns.iter().map(|pattern| GlobPattern::new(pattern)).collect()
}

pub(super) fn path_to_slash(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().replace('\\', "/")
}

pub(super) fn natural_compare(left: &str, right: &str) -> Ordering {
    let left = left.as_bytes();
    let right = right.as_bytes();
    let mut li = 0;
    let mut ri = 0;

    while li < left.len() && ri < right.len() {
        if left[li].is_ascii_digit() && right[ri].is_ascii_digit() {
            let ordering = compare_number_run(left, right, &mut li, &mut ri);
            if ordering != Ordering::Equal {
                return ordering;
            }
        } else {
            let lb = left[li].to_ascii_lowercase();
            let rb = right[ri].to_ascii_lowercase();
            if lb != rb {
                return lb.cmp(&rb);
            }
            li += 1;
            ri += 1;
        }
    }

    left.len().cmp(&right.len())
}

fn matches_glob_segments(pattern: &[String], path: &[&str]) -> bool {
    if pattern.is_empty() {
        return path.is_empty();
    }

    if pattern[0] == "**" {
        if pattern.len() == 1 {
            return true;
        }
        return (0..=path.len()).any(|index| matches_glob_segments(&pattern[1..], &path[index..]));
    }

    if path.is_empty() {
        return false;
    }

    matches_glob_segment(&pattern[0], path[0]) && matches_glob_segments(&pattern[1..], &path[1..])
}

fn matches_glob_segment(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let mut p = 0;
    let mut v = 0;
    let mut star = None;
    let mut star_value = 0;

    while v < value.len() {
        if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == value[v]) {
            p += 1;
            v += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            p += 1;
            star_value = v;
        } else if let Some(star_index) = star {
            p = star_index + 1;
            star_value += 1;
            v = star_value;
        } else {
            return false;
        }
    }

    pattern[p..].iter().all(|byte| *byte == b'*')
}

fn compare_number_run(left: &[u8], right: &[u8], li: &mut usize, ri: &mut usize) -> Ordering {
    let left_start = *li;
    let right_start = *ri;
    while *li < left.len() && left[*li].is_ascii_digit() {
        *li += 1;
    }
    while *ri < right.len() && right[*ri].is_ascii_digit() {
        *ri += 1;
    }

    let left_digits = trim_leading_zeroes(&left[left_start..*li]);
    let right_digits = trim_leading_zeroes(&right[right_start..*ri]);
    left_digits
        .len()
        .cmp(&right_digits.len())
        .then_with(|| left_digits.cmp(right_digits))
        .then_with(|| (left_start..*li).len().cmp(&(right_start..*ri).len()))
}

fn trim_leading_zeroes(digits: &[u8]) -> &[u8] {
    let trimmed =
        digits.iter().position(|byte| *byte != b'0').map_or(&[][..], |index| &digits[index..]);
    if trimmed.is_empty() {
        b"0"
    } else {
        trimmed
    }
}
