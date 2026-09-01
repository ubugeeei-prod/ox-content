pub(crate) fn camel(id: &str) -> String {
    let mut output = String::with_capacity(id.len());
    let mut uppercase_next = false;

    for character in id.chars() {
        if character == '-' {
            uppercase_next = true;
            continue;
        }
        if uppercase_next {
            output.extend(character.to_uppercase());
            uppercase_next = false;
        } else {
            output.push(character);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::camel;

    #[test]
    fn camel_cases_theme_ids() {
        assert_eq!(camel("liquid-glass"), "liquidGlass");
        assert_eq!(camel("github-high-contrast"), "githubHighContrast");
    }
}
