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

    // Normalize the text
    let text = text.trim().to_lowercase();

    // Match ASCII parts against banned text
    if ILLEGAL_TEXT
        .iter()
        .any(|illegal| contains_smart(&text, illegal))
    {
        // TODO: ensure this implementation is fixed, and start returning true again
        warn!("Found illegal text");
        error!("Bypassing ban for illegal text! Preventing possible false positive. Please check this.");
        return false;
    }

    false
}

/// Smart check whehter two strings match.
///
/// This compares characters, ignoring case.
/// All non ASCII chars are considered to be equal, to bypass text obfuscation.
///
/// The `contains` value should only contain ASCII characters.
// TODO: find a better way of doing this, this issued a faulty audit in the past
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
        .take(text_len - contains_len + 1)
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
                // At least 20% and at least 4 must be ASCII to be valid
                let set_len = set.clone().filter(|c| !c.is_whitespace()).count();
                let min_ascii = (1 + set_len / 5).max(set_len.min(4));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_smart() {
        assert!(contains_smart("a", "a"));
        assert!(contains_smart("aa", "a"));
        assert!(contains_smart("aaaaaaa", "a"));
        assert!(contains_smart("aaaaaaa", "aaa"));
        assert!(!contains_smart("a", "aa"));
        assert!(contains_smart("abcdefg", "c"));
        assert!(contains_smart("abcdefg", "bc"));
        assert!(contains_smart("abcdefg", "g"));
        assert!(!contains_smart("     ", " "));

        assert!(!contains_smart("éééééééééééééééééééé", "e"));
        assert!(!contains_smart("éééééééééééééééééééé", "é"));
        assert!(contains_smart("éééééeeééééééééééééé", "eé"));
        assert!(!contains_smart("éééééabcéééééééé", "abcdefghijkl"));
        assert!(contains_smart("éééééabcdéééééééé", "abcdefghijkl"));
        assert!(!contains_smart("éééééabcdéééééééé", "abcdefghijklm"));
        assert!(!contains_smart("éééééabcdéééééééé", "bcdefghijkl"));
        assert!(contains_smart("thís ís sómé tést", "this is some test"));
        assert!(!contains_smart("thís ís sómé tést", "this"));

        assert!(contains_smart("Celebrating our new crypto exchange", "Celebrating Our New Crypto Exchange"));
        assert!(!contains_smart("Celebrating our old crypto exchange", "Celebrating Our New Crypto Exchange"));

        assert!(contains_smart("Вinаⴖce US", "Binance US"));
        assert!(contains_smart("Βἱnаⴖcе US", "Binance US"));
        assert!(contains_smart("𐌉МΡOR𐌕АΝΤAA", "IMPORTANTAA"));
        assert!(contains_smart("ⵏMР𐩒RΤΑNТAA", "IMPORTANTAA"));
        assert!(!contains_smart("𐌉МΡOR𐌕АΝΤA", "IMPORTANTA"));
        assert!(contains_smart("ⴹνеⴖt еnd𐑈 tоԁау!", "Event ends today!"));
        assert!(contains_smart("Ενеⴖt ends tоԁаγ!", "Event ends today!"));
        assert!(contains_smart("𐌏ⴖƖγ thе fіr𐑈t 5000 u𐑈егs wἱƖƖ be гewardеd", "Only the first 5000 users will be rewarded"));
        assert!(contains_smart("OnƖy the fἰгѕt 5000 u𐑈егѕ ԝіlƖ ƅe reԝаrdеd", "Only the first 5000 users will be rewarded"));

        // Historical false positives
        assert!(!contains_smart("Oh ja tuurlijk, sancties. 🤦🏻‍♂️🤦🏻‍♂️🤦🏻‍♂️🤦🏻‍♂️", "Celebrating Our New Crypto Exchange"));
    }
}
