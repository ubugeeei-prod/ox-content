const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] =
    ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

pub type ParsedDate = (i32, u32, u32, u32, u32, u32);

pub fn parse_date(value: Option<&str>) -> Option<ParsedDate> {
    let value = value.map(str::trim).filter(|value| !value.is_empty())?;
    if value.len() < 10
        || value.as_bytes().get(4) != Some(&b'-')
        || value.as_bytes().get(7) != Some(&b'-')
    {
        return None;
    }
    let year: i32 = value.get(0..4)?.parse().ok()?;
    let month: u32 = value.get(5..7)?.parse().ok()?;
    let day: u32 = value.get(8..10)?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let (hour, minute, second) = parse_time(value).unwrap_or((0, 0, 0));
    Some((year, month, day, hour, minute, second))
}

fn parse_time(value: &str) -> Option<(u32, u32, u32)> {
    if value.len() < 19 {
        return None;
    }
    let sep = value.as_bytes()[10];
    if sep != b'T' && sep != b' ' {
        return None;
    }
    let hour: u32 = value.get(11..13)?.parse().ok()?;
    let minute: u32 = value.get(14..16)?.parse().ok()?;
    let second: u32 = value.get(17..19)?.parse().ok()?;
    (hour <= 23 && minute <= 59 && second <= 60).then_some((hour, minute, second))
}

pub fn format_rfc3339(date: ParsedDate) -> String {
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", date.0, date.1, date.2, date.3, date.4, date.5)
}

pub fn format_rfc822(date: ParsedDate) -> String {
    format!(
        "{}, {:02} {} {:04} {:02}:{:02}:{:02} +0000",
        WEEKDAYS[weekday(date.0, date.1, date.2)],
        date.2,
        MONTHS[(date.1 - 1) as usize],
        date.0,
        date.3,
        date.4,
        date.5
    )
}

fn weekday(year: i32, month: u32, day: u32) -> usize {
    const TABLE: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let year = if month < 3 { year - 1 } else { year };
    let Some(day) = i32::try_from(day).ok() else {
        return 0;
    };
    let month_index = usize::try_from(month.saturating_sub(1)).unwrap_or(0);
    let offset = TABLE.get(month_index).copied().unwrap_or(0);
    usize::try_from((year + year / 4 - year / 100 + year / 400 + offset + day).rem_euclid(7))
        .unwrap_or(0)
}
