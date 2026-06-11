use std::fmt::Write as _;
use std::time::Duration;

pub(super) fn fmt_duration(d: Duration) -> String {
    let ns = d.as_nanos();
    if ns < 1_000 {
        let mut out = String::with_capacity(24);
        push_u128(&mut out, ns);
        out.push_str(" ns");
        out
    } else if ns < 1_000_000 {
        let mut out = String::new();
        push_fmt(&mut out, format_args!("{:.2} µs", ns as f64 / 1_000.0));
        out
    } else if ns < 1_000_000_000 {
        let mut out = String::new();
        push_fmt(&mut out, format_args!("{:.2} ms", ns as f64 / 1_000_000.0));
        out
    } else {
        let mut out = String::new();
        push_fmt(&mut out, format_args!("{:.3} s", d.as_secs_f64()));
        out
    }
}

pub(super) fn fmt_bytes(b: u64) -> String {
    fmt_bytes_f(b as f64)
}

pub(super) fn fmt_bytes_f(b: f64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut value = b;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    let mut out = String::new();
    if unit == 0 {
        push_fmt(&mut out, format_args!("{value:.0} {}", UNITS[unit]));
    } else {
        push_fmt(&mut out, format_args!("{value:.2} {}", UNITS[unit]));
    }
    out
}

pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let mut t = String::with_capacity(max);
        t.push_str(&s[..max.saturating_sub(1)]);
        t.push('…');
        t
    }
}

pub(super) fn write_kv_str(s: &mut String, k: &str, v: &str) {
    s.push('"');
    s.push_str(k);
    s.push_str("\":\"");
    for ch in v.chars() {
        match ch {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            c if (c as u32) < 0x20 => push_json_control_escape(s, c as u32),
            c => s.push(c),
        }
    }
    s.push('"');
}

pub(super) fn write_kv_u64(s: &mut String, k: &str, v: u64) {
    s.push('"');
    s.push_str(k);
    s.push_str("\":");
    s.push_str(&v.to_string());
}

pub(super) fn write_kv_dur(s: &mut String, k: &str, v: Duration) {
    write_kv_u64(s, k, v.as_nanos() as u64);
}

pub(super) fn push_fmt(output: &mut String, args: std::fmt::Arguments<'_>) {
    if output.write_fmt(args).is_err() {
        output.push_str("[formatting failed]");
    }
}

fn push_u128(output: &mut String, value: u128) {
    let mut buffer = [0_u8; 39];
    let mut cursor = buffer.len();
    let mut rest = value;

    loop {
        cursor -= 1;
        buffer[cursor] = b'0' + (rest % 10) as u8;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }

    let digits = std::str::from_utf8(&buffer[cursor..]).expect("digits are valid utf-8");
    output.push_str(digits);
}

fn push_json_control_escape(output: &mut String, value: u32) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    output.push_str("\\u00");
    let low = (value & 0xff) as usize;
    output.push(char::from(HEX[(low >> 4) & 0xf]));
    output.push(char::from(HEX[low & 0xf]));
}
