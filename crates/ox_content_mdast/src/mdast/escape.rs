//! JSON scalar serialization helpers shared by the mdast serializer:
//! chunk-scanned string escaping and allocation-free integer writing.

/// Writes `value` into `output` as a quoted JSON string, escaping the
/// two structural bytes (`"`, `\`) and every control byte below 0x20.
pub(super) fn write_json_string(output: &mut String, value: &str) {
    output.push('"');
    let bytes = value.as_bytes();
    let mut start = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        // 8-byte chunk fast-skip over bytes that never need escaping
        // (everything except `"`, `\`, and control bytes < 0x20) —
        // the common case for prose, URLs, and code values. Same
        // shape as the renderer's HTML escape scan.
        while i + 8 <= bytes.len() {
            let chunk = &bytes[i..i + 8];
            let mask = JSON_ESCAPE_FLAG[chunk[0] as usize]
                | JSON_ESCAPE_FLAG[chunk[1] as usize]
                | JSON_ESCAPE_FLAG[chunk[2] as usize]
                | JSON_ESCAPE_FLAG[chunk[3] as usize]
                | JSON_ESCAPE_FLAG[chunk[4] as usize]
                | JSON_ESCAPE_FLAG[chunk[5] as usize]
                | JSON_ESCAPE_FLAG[chunk[6] as usize]
                | JSON_ESCAPE_FLAG[chunk[7] as usize];
            if mask != 0 {
                break;
            }
            i += 8;
        }
        while i < bytes.len() && JSON_ESCAPE_FLAG[bytes[i] as usize] == 0 {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }

        if start < i {
            output.push_str(&value[start..i]);
        }
        let byte = bytes[i];
        match byte {
            b'"' => output.push_str("\\\""),
            b'\\' => output.push_str("\\\\"),
            b'\n' => output.push_str("\\n"),
            b'\r' => output.push_str("\\r"),
            b'\t' => output.push_str("\\t"),
            b'\x08' => output.push_str("\\b"),
            b'\x0c' => output.push_str("\\f"),
            _ => push_json_byte_escape(output, byte),
        }
        i += 1;
        start = i;
    }

    if start < value.len() {
        output.push_str(&value[start..]);
    }
    output.push('"');
}

/// Writes a decimal integer without the `to_string` heap allocation
/// the previous implementation paid per heading depth / list start.
pub(super) fn write_u32(output: &mut String, mut n: u32) {
    let mut buf = [0u8; 10];
    let mut at = buf.len();
    loop {
        at -= 1;
        buf[at] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    for &digit in &buf[at..] {
        output.push(char::from(digit));
    }
}

/// `JSON_ESCAPE_FLAG[b] == 1` iff `b` must be escaped inside a JSON
/// string: the two structural bytes and every control byte below 0x20.
static JSON_ESCAPE_FLAG: [u8; 256] = {
    let mut t = [0u8; 256];
    let mut b = 0usize;
    while b < 0x20 {
        t[b] = 1;
        b += 1;
    }
    t[b'"' as usize] = 1;
    t[b'\\' as usize] = 1;
    t
};

fn push_json_byte_escape(output: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    output.push_str("\\u00");
    output.push(char::from(HEX[usize::from((byte >> 4) & 0x0f)]));
    output.push(char::from(HEX[usize::from(byte & 0x0f)]));
}
