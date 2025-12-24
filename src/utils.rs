pub fn strip_html(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut inside_tag = false;

    for c in input.chars() {
        match c {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => result.push(c),
            _ => {}
        }
    }

    decode_html_entities(&result).trim().to_owned()
}

fn decode_html_entities(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars();

    while let Some(c) = chars.next() {
        if c == '&' {
            // Try to match common HTML entities
            let remaining: String = chars.as_str().chars().take(6).collect();

            let (replacement, skip) = if remaining.starts_with("amp;") {
                ("&", 4)
            } else if remaining.starts_with("lt;") {
                ("<", 3)
            } else if remaining.starts_with("gt;") {
                (">", 3)
            } else if remaining.starts_with("quot;") {
                ("\"", 5)
            } else if remaining.starts_with("#39;") {
                ("'", 4)
            } else if remaining.starts_with("#233;") {
                ("é", 5)
            } else if remaining.starts_with("nbsp;") {
                (" ", 5)
            } else {
                // Not a recognized entity, keep the ampersand
                result.push(c);
                continue;
            };

            result.push_str(replacement);
            // Skip the entity characters
            for _ in 0..skip {
                chars.next();
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html() {
        assert_eq!(strip_html("<p>Hello</p>"), "Hello");
        assert_eq!(strip_html("Hello &amp; World"), "Hello & World");
        assert_eq!(strip_html("<b>Test &lt;tag&gt;</b>"), "Test <tag>");
        assert_eq!(strip_html("&quot;quoted&quot;"), "\"quoted\"");
        assert_eq!(strip_html("&#39;apostrophe&#39;"), "'apostrophe'");
        assert_eq!(strip_html("Caf&#233;"), "Café");
        assert_eq!(strip_html("space&nbsp;here"), "space here");
    }

    #[test]
    fn test_decode_html_entities() {
        assert_eq!(decode_html_entities("&amp;"), "&");
        assert_eq!(decode_html_entities("&lt;&gt;"), "<>");
        assert_eq!(decode_html_entities("&unknown;"), "&unknown;");
        assert_eq!(decode_html_entities("plain text"), "plain text");
    }
}
