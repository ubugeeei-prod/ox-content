mod lookup_a_d;
mod lookup_e_l;
mod lookup_m_r;
mod lookup_s_z;
mod lookup_symbols;

pub(super) fn is_shortcode_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'+')
}

pub(super) fn lookup(name: &str) -> Option<&'static str> {
    match name.as_bytes().first().copied()? {
        b'+' | b'-' | b'0'..=b'9' => lookup_symbols::lookup(name),
        b'a'..=b'd' => lookup_a_d::lookup(name),
        b'e'..=b'l' => lookup_e_l::lookup(name),
        b'm'..=b'r' => lookup_m_r::lookup(name),
        b's'..=b'z' => lookup_s_z::lookup(name),
        _ => None,
    }
}
