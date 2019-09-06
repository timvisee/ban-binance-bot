use futures::Future;

/// Check whether the given text is illegal.
pub fn is_illegal_text(text: String) -> impl Future<Item = bool, Error = ()> {
    // Check for illegal URLs
    super::url::contains_illegal_urls(&text)
}
