pub(super) fn get_trailing_whitespace_length(line: &str) -> usize {
    line.bytes().rev().take_while(|b| matches!(b, b' ' | b'\t')).count()
}
