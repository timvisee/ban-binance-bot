use crate::config::ILLEGAL_TEXT;

/// Check whether the given text is illegal.
pub async fn is_illegal_text(text: String) -> bool {
    // Check for illegal text
    if matches_illegal_text(&text) {
        return true;
    }

    // Check for illegal URLs
    super::url::contains_illegal_urls(&text).await
}

/// Check whether the text contains illegal parts.
pub fn matches_illegal_text(text: &str) -> bool {
    // We must have illegal text configured
    if ILLEGAL_TEXT.is_empty() {
        return false;
    }

    // // Normalize the text
    // let text = text.trim().to_lowercase();

    // // Match ASCII parts against banned text
    // if ILLEGAL_TEXT
    //     .iter()
    //     .any(|illegal| contains_smart(&text, illegal))
    // {
    //     warn!("Found illegal text");
    //     return true;
    // }

    false
}

/// Smart check whehter two strings match.
///
/// This compares characters, ignoring case.
/// All non ASCII chars are considered to be equal, to bypass text obfuscation.
///
/// The `contains` value should only contain ASCII characters.
fn contains_smart(text: &str, contains: &str) -> bool {
    // Both must not be empty, contains must not be shorter
    if text.is_empty() || contains.is_empty() || text.len() < contains.len() {
        return false;
    }

    // Get haystack and needle length in characters, skip if haystack not big enough
    let text_len = text.chars().count();
    let contains_len = contains.chars().count();
    if text_len < contains_len {
        return false;
    }

    // Compare
    let contains_first = contains.chars().next().unwrap();
    text
        .chars()
        .take(text_len - contains_len)
        .enumerate()
        .filter(|(_, c)| char_matches_smart(*c, contains_first))
        .any(|(i, _)| {
            // Define set to scan
            let set = text.chars().skip(i).take(contains_len);

            // Match all ASCII characters, and ensure at least 20% is ASCII
            set.clone()
                .zip(contains.chars())
                .all(|(a, b)| char_matches_smart(a, b))
            && {
                // At least 20% must be ASCII to be valid
                let text_len = set.clone().filter(|c| !c.is_whitespace()).count();
                let min_ascii = 1 + text_len / 5;
                set.filter(|c| c.is_ascii() && !c.is_whitespace()).count() >= min_ascii
            }
        })
}

/// Smart check whehter a char matches.
///
/// This compares characters, ignoring case.
/// All non ASCII chars are considered to be equal, to bypass text obfuscation.
#[inline]
fn char_matches_smart(a: char, b: char) -> bool {
    a == b || !a.is_ascii() || !b.is_ascii() || a.to_ascii_uppercase() == b.to_ascii_uppercase()
}
