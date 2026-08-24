//! Opt-in draft / unlisted / scheduled page classification.

use std::collections::HashMap;
use std::hash::BuildHasher;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

/// Plugin options for publish-state filtering.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PublishStateOptions {
    /// When false, every page stays published and listed.
    pub enabled: bool,
    /// Injected ISO-8601 clock. Invalid values fall back to system time.
    pub now: Option<String>,
    /// Dev preview: keep draft, scheduled, and expired pages in output.
    pub include_drafts: bool,
}

/// Whether a page should be written and whether it should appear in listings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublishDecision {
    /// Write HTML (and OG images) for this page.
    pub output: bool,
    /// Include the page in nav, sitemap, and search.
    pub listed: bool,
}

/// Classifies one page from already-parsed frontmatter.
///
/// Invalid `scheduled` or `expiry` values unpublish the page. Invalid `date`
/// values are ignored because `date` is also used as display metadata.
/// Naive timestamps are UTC. Only JSON `true` counts as `draft` / `unlisted`.
#[must_use]
pub fn classify_publish_state<S: BuildHasher>(
    frontmatter: &HashMap<String, Value, S>,
    options: &PublishStateOptions,
) -> PublishDecision {
    if !options.enabled {
        return PublishDecision { output: true, listed: true };
    }

    let unpublished = is_unpublished(frontmatter, options);
    let unlisted = is_flag(frontmatter, "unlisted");
    if unpublished {
        return PublishDecision { output: false, listed: false };
    }
    PublishDecision { output: true, listed: !unlisted }
}

fn is_unpublished<S: BuildHasher>(
    frontmatter: &HashMap<String, Value, S>,
    options: &PublishStateOptions,
) -> bool {
    if options.include_drafts {
        return false;
    }
    if is_flag(frontmatter, "draft") {
        return true;
    }
    let now = resolve_now(options.now.as_deref());
    if !schedule_reached(frontmatter.get("scheduled"), now, true)
        || !schedule_reached(frontmatter.get("date"), now, false)
    {
        return true;
    }
    is_expired(frontmatter.get("expiry"), now)
}

fn is_flag<S: BuildHasher>(frontmatter: &HashMap<String, Value, S>, key: &str) -> bool {
    matches!(frontmatter.get(key), Some(Value::Bool(true)))
}

fn schedule_reached(value: Option<&Value>, now: i64, invalid_unpublishes: bool) -> bool {
    match parse_frontmatter_instant(value) {
        DateField::Absent => true,
        DateField::Instant(instant) => instant <= now,
        DateField::Invalid => !invalid_unpublishes,
    }
}

fn is_expired(value: Option<&Value>, now: i64) -> bool {
    match parse_frontmatter_instant(value) {
        DateField::Absent => false,
        DateField::Instant(instant) => instant < now,
        DateField::Invalid => true,
    }
}

enum DateField {
    Absent,
    Instant(i64),
    Invalid,
}

fn parse_frontmatter_instant(value: Option<&Value>) -> DateField {
    match value {
        None | Some(Value::Null) => DateField::Absent,
        Some(Value::String(text)) if text.trim().is_empty() => DateField::Absent,
        Some(Value::String(text)) => {
            parse_iso_datetime(text.trim()).map_or(DateField::Invalid, DateField::Instant)
        }
        Some(Value::Number(number)) => number
            .as_i64()
            .and_then(unix_from_number)
            .map_or(DateField::Invalid, DateField::Instant),
        Some(_) => DateField::Invalid,
    }
}

fn unix_from_number(value: i64) -> Option<i64> {
    if (1_000_000_000..1_000_000_000_000).contains(&value) {
        Some(value)
    } else if (1_000_000_000_000..1_000_000_000_000_000).contains(&value) {
        Some(value / 1000)
    } else {
        None
    }
}

fn resolve_now(now: Option<&str>) -> i64 {
    now.and_then(parse_iso_datetime).unwrap_or_else(system_now)
}

fn system_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs().cast_signed())
}

/// Parses a subset of ISO-8601 / RFC 3339. Naive values are UTC.
pub fn parse_iso_datetime(input: &str) -> Option<i64> {
    let input = input.trim();
    if input.is_empty() || input.len() > 40 || !input.is_ascii() {
        return None;
    }
    let (date, rest) = input.split_at_checked(10)?;
    let (year, month, day) = parse_ymd(date)?;
    if rest.is_empty() {
        return civil_to_unix(year, month, day, 0, 0, 0, 0);
    }
    let rest = rest.strip_prefix('T').or_else(|| rest.strip_prefix(' '))?;
    parse_time_and_offset(year, month, day, rest)
}

fn parse_ymd(date: &str) -> Option<(i32, u32, u32)> {
    let mut parts = date.split('-');
    let year = parse_signed_width(parts.next()?, 4)?;
    let month = parse_width(parts.next()?, 2)?;
    let day = parse_width(parts.next()?, 2)?;
    if parts.next().is_some() {
        return None;
    }
    Some((year, month, day))
}

fn parse_time_and_offset(year: i32, month: u32, day: u32, rest: &str) -> Option<i64> {
    if rest.len() < 8 {
        return None;
    }
    let hour = parse_width(&rest[..2], 2)?;
    if rest.as_bytes().get(2) != Some(&b':') {
        return None;
    }
    let minute = parse_width(&rest[3..5], 2)?;
    if rest.as_bytes().get(5) != Some(&b':') {
        return None;
    }
    let second = parse_width(&rest[6..8], 2)?;
    let after_time = &rest[8..];
    let after_frac = after_time.strip_prefix('.').map_or(after_time, |frac| {
        let digits = frac.bytes().take_while(u8::is_ascii_digit).count();
        &frac[digits..]
    });
    let offset = parse_offset(after_frac)?;
    civil_to_unix(year, month, day, hour, minute, second, offset)
}

fn parse_offset(input: &str) -> Option<i32> {
    if input.is_empty() || input == "Z" || input == "z" {
        return Some(0);
    }
    let (sign, rest) = match input.as_bytes().first()? {
        b'+' => (1, &input[1..]),
        b'-' => (-1, &input[1..]),
        _ => return None,
    };
    let (hour, minute) = if rest.len() == 5 && rest.as_bytes().get(2) == Some(&b':') {
        (parse_width(&rest[..2], 2)?, parse_width(&rest[3..], 2)?)
    } else if rest.len() == 4 {
        (parse_width(&rest[..2], 2)?, parse_width(&rest[2..], 2)?)
    } else {
        return None;
    };
    if hour > 23 || minute > 59 {
        return None;
    }
    Some(sign * (hour.cast_signed() * 3600 + minute.cast_signed() * 60))
}

fn parse_width(input: &str, width: usize) -> Option<u32> {
    (input.len() == width && input.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| input.parse().ok())
        .flatten()
}

fn parse_signed_width(input: &str, width: usize) -> Option<i32> {
    parse_width(input, width)?.try_into().ok()
}

fn civil_to_unix(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    offset_secs: i32,
) -> Option<i64> {
    if !(1..=12).contains(&month)
        || hour > 23
        || minute > 59
        || second > 60
        || !(0..=9999).contains(&year)
    {
        return None;
    }
    let max_day = days_in_month(year, month)?;
    if day == 0 || day > max_day {
        return None;
    }
    let days = days_from_civil(year, month, day);
    days.checked_mul(86_400)?
        .checked_add(i64::from(hour) * 3600 + i64::from(minute) * 60 + i64::from(second))?
        .checked_sub(i64::from(offset_secs))
}

fn days_in_month(year: i32, month: u32) -> Option<u32> {
    Some(match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => return None,
    })
}

fn is_leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = i64::from(year);
    let month = i64::from(month);
    let day = i64::from(day);
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 }.div_euclid(400);
    let yoe = year - era * 400;
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests;
