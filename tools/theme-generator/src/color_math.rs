pub(crate) fn mix(color: &str, amount: u8) -> String {
    mix_into(color, amount, "transparent")
}

pub(crate) fn mix_into(color: &str, amount: u8, into: &str) -> String {
    format!("color-mix(in srgb, {color} {amount}%, {into})")
}

pub(crate) fn rgb_triplet(hex: &str) -> (u8, u8, u8) {
    let value = hex.trim_start_matches('#');
    let value = value.get(..6).unwrap_or(value);
    let parsed = u32::from_str_radix(value, 16).unwrap_or(0);

    (((parsed >> 16) & 255) as u8, ((parsed >> 8) & 255) as u8, (parsed & 255) as u8)
}

pub(crate) fn rgb_triplet_text(hex: &str) -> String {
    let (red, green, blue) = rgb_triplet(hex);
    format!("{red}, {green}, {blue}")
}

pub(crate) fn contrast(a: &str, b: &str) -> f64 {
    let (light, dark) = sorted_luminance(a, b);
    (light + 0.05) / (dark + 0.05)
}

pub(crate) fn ensure_contrast(color: &str, background: &str, target: f64) -> String {
    if contrast(color, background) >= target {
        return color.to_string();
    }

    let toward = if luminance(background) > 0.4 { 0.0 } else { 255.0 };
    let base = rgb_triplet(color);
    for step in 1..=25 {
        let ratio = f64::from(step) * 0.04;
        let mixed = [
            to_hex((toward - f64::from(base.0)).mul_add(ratio, f64::from(base.0))),
            to_hex((toward - f64::from(base.1)).mul_add(ratio, f64::from(base.1))),
            to_hex((toward - f64::from(base.2)).mul_add(ratio, f64::from(base.2))),
        ]
        .join("");
        let candidate = format!("#{mixed}");
        if contrast(&candidate, background) >= target {
            return candidate;
        }
    }

    if toward == 0.0 { "#000000" } else { "#ffffff" }.to_string()
}

pub(crate) fn backdrop(hex: &str) -> String {
    let (red, green, blue) = rgb_triplet(hex);
    format!(
        "rgba({}, {}, {}, 0.62)",
        scaled_backdrop(red),
        scaled_backdrop(green),
        scaled_backdrop(blue)
    )
}

fn sorted_luminance(a: &str, b: &str) -> (f64, f64) {
    let first = luminance(a);
    let second = luminance(b);
    if first >= second { (first, second) } else { (second, first) }
}

fn luminance(hex: &str) -> f64 {
    let channels = <[u8; 3]>::from(rgb_triplet(hex)).map(|channel| {
        let value = f64::from(channel) / 255.0;
        if value <= 0.03928 { value / 12.92 } else { ((value + 0.055) / 1.055).powf(2.4) }
    });

    channels[0].mul_add(0.2126, channels[1].mul_add(0.7152, 0.0722 * channels[2]))
}

fn to_hex(value: f64) -> String {
    let rounded = value.round().clamp(0.0, 255.0) as u8;
    format!("{rounded:02x}")
}

fn scaled_backdrop(channel: u8) -> u8 {
    (f64::from(channel) * 0.35).round() as u8
}

#[cfg(test)]
mod tests {
    use super::{contrast, ensure_contrast, rgb_triplet_text};

    #[test]
    fn formats_rgb_triplets_like_css_variables_expect() {
        assert_eq!(rgb_triplet_text("#8250df"), "130, 80, 223");
    }

    #[test]
    fn nudges_accents_until_they_clear_aa_contrast() {
        let color = ensure_contrast("#79b8ff", "#ffffff", 4.5);
        assert!(contrast(&color, "#ffffff") >= 4.5);
    }
}
