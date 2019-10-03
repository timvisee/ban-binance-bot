/// Check whether the given text is illegal.
pub async fn is_illegal_text(text: String) -> Result<bool, ()> {
    // Check for illegal URLs
    super::url::contains_illegal_urls(&text).await
}
