use super::parse::TimelineDate;
use super::{ReportMode, ResolvedTimelineOptions};

pub(super) fn parse_date_and_title<'a>(
    value: &'a str,
    line: usize,
    options: &ResolvedTimelineOptions,
    diagnostics: &mut Vec<String>,
) -> (Option<TimelineDate>, &'a str) {
    if let Some(rest) = value.strip_prefix('[')
        && let Some(end) = rest.find(']')
    {
        let date = parse_date_token(&rest[..end], line, options, diagnostics);
        return (date, rest[end + 1..].trim_start());
    }
    let Some((token, rest)) = split_first_token(value) else {
        return (None, value);
    };
    if !is_date_candidate(token) {
        return (None, value);
    }
    (parse_date_token(token, line, options, diagnostics), rest.trim_start())
}

fn parse_date_token(
    token: &str,
    line: usize,
    options: &ResolvedTimelineOptions,
    diagnostics: &mut Vec<String>,
) -> Option<TimelineDate> {
    if is_valid_date(token) {
        return Some(TimelineDate { text: token.to_string(), datetime: Some(token.to_string()) });
    }
    let message = format!("Timeline item on line {line} has invalid date `{token}`.");
    match options.invalid_date {
        ReportMode::Ignore => None,
        ReportMode::Warn => {
            diagnostics.push(message);
            Some(TimelineDate { text: token.to_string(), datetime: None })
        }
        ReportMode::Error => {
            diagnostics.push(format!("error:{message}"));
            Some(TimelineDate { text: token.to_string(), datetime: None })
        }
    }
}

fn split_first_token(value: &str) -> Option<(&str, &str)> {
    let trimmed = value.trim_start();
    if trimmed.is_empty() {
        return None;
    }
    let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
    Some((&trimmed[..end], &trimmed[end..]))
}

fn is_date_candidate(value: &str) -> bool {
    value.len() >= 4 && value.chars().all(|ch| ch.is_ascii_digit() || ch == '-')
}

fn is_valid_date(value: &str) -> bool {
    match value.len() {
        4 => value.parse::<u16>().is_ok_and(|year| year > 0),
        7 => parse_year_month(value).is_some(),
        10 => {
            let Some((year, month)) = parse_year_month(&value[..7]) else {
                return false;
            };
            if value.as_bytes().get(7) != Some(&b'-') {
                return false;
            }
            value[8..].parse::<u8>().is_ok_and(|day| day > 0 && day <= days_in_month(year, month))
        }
        _ => false,
    }
}

fn parse_year_month(value: &str) -> Option<(u16, u8)> {
    if value.as_bytes().get(4) != Some(&b'-') {
        return None;
    }
    let year = value[..4].parse::<u16>().ok().filter(|year| *year > 0)?;
    let month = value[5..].parse::<u8>().ok().filter(|month| (1..=12).contains(month))?;
    Some((year, month))
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && !year.is_multiple_of(100) || year.is_multiple_of(400)
}
