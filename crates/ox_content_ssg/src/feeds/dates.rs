//! Civil-date parsing and RFC 822 / RFC 3339 formatting for feeds.

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] =
    ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

#[derive(Clone, Copy)]
pub(super) struct ParsedDate {
    unix: i64,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

impl ParsedDate {
    pub(super) fn unix(self) -> i64 {
        self.unix
    }

    pub(super) fn rfc3339(self) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }

    pub(super) fn rfc822(self) -> String {
        format!(
            "{}, {:02} {} {:04} {:02}:{:02}:{:02} +0000",
            WEEKDAYS[weekday_utc(self.year, self.month, self.day)],
            self.day,
            MONTHS[usize::try_from(self.month.saturating_sub(1)).unwrap_or(0)],
            self.year,
            self.hour,
            self.minute,
            self.second
        )
    }
}

pub(super) fn parse_date(value: Option<&str>) -> Option<ParsedDate> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }
    if value.bytes().all(|byte| byte.is_ascii_digit()) {
        let n: i64 = value.parse().ok()?;
        let unix = if value.len() >= 13 { n.div_euclid(1000) } else { n };
        return unix_to_date(unix);
    }
    parse_civil_date(value)
}

fn parse_civil_date(value: &str) -> Option<ParsedDate> {
    let bytes = value.as_bytes();
    if bytes.len() < 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let year: i32 = std::str::from_utf8(&bytes[0..4]).ok()?.parse().ok()?;
    let month: u32 = std::str::from_utf8(&bytes[5..7]).ok()?.parse().ok()?;
    let day: u32 = std::str::from_utf8(&bytes[8..10]).ok()?.parse().ok()?;
    let (hour, minute, second, offset) = if bytes.len() > 10 {
        let rest = value.get(10..)?;
        let rest = rest.strip_prefix('T').or_else(|| rest.strip_prefix(' '))?;
        let rest_bytes = rest.as_bytes();
        if rest_bytes.len() < 8 || rest_bytes[2] != b':' || rest_bytes[5] != b':' {
            return None;
        }
        (
            std::str::from_utf8(&rest_bytes[0..2]).ok()?.parse().ok()?,
            std::str::from_utf8(&rest_bytes[3..5]).ok()?.parse().ok()?,
            std::str::from_utf8(&rest_bytes[6..8]).ok()?.parse().ok()?,
            parse_offset(timezone_suffix(rest))?,
        )
    } else {
        (0, 0, 0, 0)
    };
    unix_to_date(civil_to_unix(year, month, day, hour, minute, second)? - offset)
}

fn timezone_suffix(rest: &str) -> &str {
    let after_time = rest.get(8..).unwrap_or("");
    after_time.strip_prefix('.').map_or(after_time, |stripped| {
        stripped.find(['Z', '+', '-']).map_or("", |index| &stripped[index..])
    })
}

fn parse_offset(tz: &str) -> Option<i64> {
    if tz.is_empty() || tz == "Z" {
        return Some(0);
    }
    let bytes = tz.as_bytes();
    if bytes.len() < 6 {
        return None;
    }
    let sign: i64 = match bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let hour: i64 = std::str::from_utf8(&bytes[1..3]).ok()?.parse().ok()?;
    let minute: i64 = std::str::from_utf8(&bytes[4..6]).ok()?.parse().ok()?;
    Some(sign * (hour * 3_600 + minute * 60))
}

fn civil_to_unix(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Option<i64> {
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }
    let month = i64::from(month);
    let mut year = i64::from(year);
    if month <= 2 {
        year -= 1;
    }
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let shifted = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * shifted + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * 86_400 + i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second))
}

fn unix_to_date(unix: i64) -> Option<ParsedDate> {
    let days = unix.div_euclid(86_400);
    let tod = unix.rem_euclid(86_400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    Some(ParsedDate {
        unix,
        year: i32::try_from(year + i64::from(month <= 2)).ok()?,
        month: u32::try_from(month).ok()?,
        day: u32::try_from(day).ok()?,
        hour: u32::try_from(tod / 3_600).ok()?,
        minute: u32::try_from((tod % 3_600) / 60).ok()?,
        second: u32::try_from(tod % 60).ok()?,
    })
}

fn weekday_utc(year: i32, month: u32, day: u32) -> usize {
    const TABLE: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let year = if month < 3 { year - 1 } else { year };
    let month_index = usize::try_from(month.saturating_sub(1)).unwrap_or(0);
    let raw = year + year / 4 - year / 100
        + year / 400
        + TABLE.get(month_index).copied().unwrap_or(0)
        + i32::try_from(day).unwrap_or(0);
    usize::try_from(raw.rem_euclid(7)).unwrap_or(0)
}
