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

pub fn normalize_title_name(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut prev_was_dash = false;

    for ch in input.chars() {
        match ch {
            ' ' => {
                if !result.is_empty() && !prev_was_dash {
                    result.push('-');
                    prev_was_dash = true;
                }
            }
            '®' => {
                // result.push('r');
                prev_was_dash = false;
            }
            '™' => {
                prev_was_dash = false;
            }
            'é' | 'É' => {
                result.push('e');
                prev_was_dash = false;
            }
            c if c.is_ascii_alphanumeric() || c == '-' => {
                let lower = c.to_ascii_lowercase();
                if lower == '-' && (result.is_empty() || prev_was_dash) {
                    continue;
                }
                result.push(lower);
                prev_was_dash = lower == '-';
            }
            _ => {}
        }
    }

    if result.ends_with('-') {
        result.pop();
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

    #[test]
    fn test_normalize_title_name() {
        assert_eq!(
            normalize_title_name("Super Mario Odyssey"),
            "super-mario-odyssey"
        );
        assert_eq!(
            normalize_title_name("Pokémon Mystery Dungeon: Rescue Team DX"),
            "pokemon-mystery-dungeon-rescue-team-dx"
        );
        assert_eq!(
            normalize_title_name("Mario Kart™ 8 Deluxe"),
            "mario-kart-8-deluxe"
        );
        assert_eq!(
            normalize_title_name("Xenoblade Chronicles® 2"),
            "xenoblade-chroniclesr-2"
        );
        assert_eq!(
            normalize_title_name("The Legend of Zelda: Breath of the Wild"),
            "the-legend-of-zelda-breath-of-the-wild"
        );
        assert_eq!(
            normalize_title_name("Super Smash Bros.™ Ultimate"),
            "super-smash-bros-ultimate"
        );
    }
}
