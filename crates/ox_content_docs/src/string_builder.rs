/// Small append-only string builder used by documentation generation hot paths.
///
/// This wraps `String` so call sites can express "append a few known pieces"
/// without reaching for `format!` or `to_string()` for small numeric fragments.
/// The helper methods keep the final output allocation explicit via
/// `with_capacity`, and `push_usize` writes decimal digits through a stack
/// buffer so count-heavy renderers do not allocate temporary strings.
pub struct StringBuilder {
    output: String,
}

impl StringBuilder {
    pub fn new() -> Self {
        Self { output: String::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { output: String::with_capacity(capacity) }
    }

    pub fn push_str(&mut self, value: &str) {
        self.output.push_str(value);
    }

    pub fn push_char(&mut self, value: char) {
        self.output.push(value);
    }

    pub fn push_usize(&mut self, value: usize) {
        // Maximum decimal length of `usize` on supported targets is 20 bytes
        // (`u64::MAX`). Fill the stack buffer from the back, then append the
        // valid suffix in one `push_str`; this replaces `value.to_string()`
        // in loops that render stats, anchors, headings, and file names.
        let mut buffer = [0_u8; 20];
        push_decimal_digits(&mut self.output, value as u128, &mut buffer);
    }

    #[cfg(test)]
    pub fn push_u128(&mut self, value: u128) {
        // Test-only wide variant for boundary coverage of the digit writer.
        // `u128::MAX` is 39 decimal digits.
        let mut buffer = [0_u8; 39];
        push_decimal_digits(&mut self.output, value, &mut buffer);
    }

    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }

    pub fn into_string(self) -> String {
        self.output
    }
}

pub fn join2(first: &str, second: &str) -> String {
    // These tiny join helpers replace `format!("{a}{b}...")` in render loops.
    // They pre-size exactly for the literal pieces and avoid the formatting
    // machinery when no formatting is needed.
    let mut out = StringBuilder::with_capacity(first.len() + second.len());
    out.push_str(first);
    out.push_str(second);
    out.into_string()
}

pub fn join3(first: &str, second: &str, third: &str) -> String {
    let mut out = StringBuilder::with_capacity(first.len() + second.len() + third.len());
    out.push_str(first);
    out.push_str(second);
    out.push_str(third);
    out.into_string()
}

pub fn join4(first: &str, second: &str, third: &str, fourth: &str) -> String {
    let mut out =
        StringBuilder::with_capacity(first.len() + second.len() + third.len() + fourth.len());
    out.push_str(first);
    out.push_str(second);
    out.push_str(third);
    out.push_str(fourth);
    out.into_string()
}

pub fn join5(first: &str, second: &str, third: &str, fourth: &str, fifth: &str) -> String {
    let mut out = StringBuilder::with_capacity(
        first.len() + second.len() + third.len() + fourth.len() + fifth.len(),
    );
    out.push_str(first);
    out.push_str(second);
    out.push_str(third);
    out.push_str(fourth);
    out.push_str(fifth);
    out.into_string()
}

fn push_decimal_digits(output: &mut String, mut value: u128, buffer: &mut [u8]) {
    // Caller-sized stack buffers are filled only with ASCII `'0'..='9'`.
    // Interpret that suffix as UTF-8, and copy bytes as chars if the check
    // ever fails rather than aborting a docs render.
    if buffer.is_empty() {
        return;
    }
    let mut cursor = buffer.len();
    loop {
        cursor -= 1;
        buffer[cursor] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 || cursor == 0 {
            break;
        }
    }
    match std::str::from_utf8(&buffer[cursor..]) {
        Ok(digits) => output.push_str(digits),
        Err(_) => output.extend(buffer[cursor..].iter().copied().map(char::from)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_usize_writes_zero_and_wide_values() {
        let mut out = StringBuilder::new();
        out.push_usize(0);
        out.push_char(' ');
        out.push_usize(10);
        out.push_char(' ');
        out.push_usize(usize::MAX);
        assert_eq!(out.into_string(), format!("0 10 {}", usize::MAX));
    }

    #[test]
    fn push_u128_writes_max_without_panic() {
        let mut out = StringBuilder::new();
        out.push_u128(0);
        out.push_char('-');
        out.push_u128(u128::MAX);
        assert_eq!(out.into_string(), format!("0-{}", u128::MAX));
    }

    #[test]
    fn joins_preserve_empty_and_hostile_fragments() {
        assert_eq!(join2("", "x"), "x");
        assert_eq!(join3("<", "script", ">"), "<script>");
        assert_eq!(join4("a", "\0", "b", "\u{1F680}"), "a\0b\u{1F680}");
        assert_eq!(join5("", "", "", "", ""), "");
    }
}
