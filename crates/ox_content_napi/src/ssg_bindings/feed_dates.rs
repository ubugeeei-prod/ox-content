use napi_derive::napi;

/// A feed date after parsing: the instant, its civil parts in UTC, and the two
/// serializations the feed formats ask for.
#[napi(object)]
pub struct JsParsedFeedDate {
    /// Seconds since the Unix epoch.
    pub unix: i64,
    /// Civil year in UTC.
    pub year: i32,
    /// Civil month, 1-12.
    pub month: u32,
    /// Civil day of month, 1-31.
    pub day: u32,
    /// Hour, 0-23.
    pub hour: u32,
    /// Minute, 0-59.
    pub minute: u32,
    /// Second, 0-60.
    pub second: u32,
    /// `YYYY-MM-DDTHH:MM:SSZ`, as Atom and JSON Feed want it.
    pub rfc3339: String,
    /// `Day, DD Mon YYYY HH:MM:SS +0000`, as RSS wants it.
    pub rfc822: String,
}

/// Parses a feed date — an epoch seconds/milliseconds integer, or a civil
/// date-time with an optional zone offset — returning `null` when the value
/// names no instant.
#[napi(js_name = "parseFeedDate")]
pub fn parse_feed_date(value: Option<String>) -> Option<JsParsedFeedDate> {
    let parsed = ox_content_ssg::parse_date(value.as_deref())?;
    let (year, month, day, hour, minute, second) = parsed.parts();
    Some(JsParsedFeedDate {
        unix: parsed.unix(),
        year,
        month,
        day,
        hour,
        minute,
        second,
        rfc3339: parsed.rfc3339(),
        rfc822: parsed.rfc822(),
    })
}
