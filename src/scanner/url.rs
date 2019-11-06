use std::time::Duration;

use futures::future::{BoxFuture, FutureExt};
use reqwest::{header, r#async::Client, RedirectPolicy};
use url::Url;

use crate::{
    config::*,
    util::{self, future::select_true, url::find_page_urls},
};

/// Check whether the given text contains any illegal URLs.
///
/// This uses `ILLEGAL_HOSTS`.
pub async fn contains_illegal_urls(text: &str) -> bool {
    // Find URLs in the message, return if there are none
    let urls = util::url::find_urls(text);
    if urls.is_empty() {
        return false;
    }

    any_illegal_url(urls, 0).await
}

/// Check whether the given list of URLs contains any illegal URL.
///
/// This uses `ILLEGAL_HOSTS`.
pub fn any_illegal_url<'a, I>(urls: I, depth: usize) -> BoxFuture<'a, bool>
where
    I: IntoIterator<Item = Url> + Send + 'a,
    I::IntoIter: Send,
{
    async move {
        // Test each URL concurrently
        select_true(urls.into_iter().map(|url| is_illegal_url(url, depth))).await
    }.boxed()
}

/// Check whether the given URL is illegal.
///
/// This compares the given URL, and the URL it possibly redirects to.
///
/// Returns `Ok` if the URL is illegal, `Err` otherwise.
/// Errors are silently dropped and it will then be assumed that the URL is allowed.
/// This allows the use of `futures::future::select_ok`.
async fn is_illegal_url(mut url: Url, depth: usize) -> bool {
    // The given URL must not be illegal
    if is_illegal_static_url(&url) {
        return true;
    }

    // Follow URL redirects
    match util::url::follow_url(&url).await {
        Ok(ref url) if is_illegal_static_url(url) => return true,
        Ok(new) => url = new,
        Err(err) => debug!("Failed to follow URL redirects, could not audit, assuming safe: {:?}", err),
    }

    // Check whether the webpage contains illegal content
    if url_has_illegal_webpage_content(&url, depth).await {
        warn!("Found illegal URL, webpage has illegal content: {}", url);
        return true;
    }

    false
}

/// Check whether the given URL routes to illegal content.
///
/// This scans the body of the webpage that is responded with.
async fn url_has_illegal_webpage_content(url: &Url, depth: usize) -> bool {
    // We must have illegal webpage text configured
    if ILLEGAL_WEBPAGE_TEXT.is_empty() {
        return false;
    }

    // Build the URL client
    // TODO: use a global client instance
    let mut headers = header::HeaderMap::new();
    headers.insert(header::COOKIE, header::HeaderValue::from_static("__test=bda194efef091b052793e3eb74b1b952; id=185"));
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(RedirectPolicy::limited(25))
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(20))
        .default_headers(headers)
        .build()
        .expect("failed to build webpage body auditer client");

    // Send the request, follow the URL
    // TODO: validate status !response.status.is_success()
    let response = match client.get(url.as_str()).send().await {
        Ok(response) => response,
        Err(err) => {
            debug!("Failed to request webpage content, could not audit, assuming safe: {}", err);
            return false;
        },
    };

    // Request the page body
    let body = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(err) => {
            warn!("Failed to receive webpage content, could not audit, assuming safe: {}", err);
            return false;
        },
    };

    // Find the shortest needle to limit body searching
    let needles = ILLEGAL_WEBPAGE_TEXT;
    let shortest = needles.iter().map(|t| t.len()).min().unwrap_or(0);

    // The body must be long enough
    if body.len() < shortest {
        debug!("Webpage body is too small to scan");
        return false;
    }

    // Scan body for needles to detect illegal content
    let has_illegal_text = (0..=body.len() - shortest)
        .any(|i| needles
            .iter()
            .filter(|needle| needle.as_bytes().len() <= body.len() - i)
            .any(|needle| if &body[i..i + needle.len()] == needle.as_bytes() {
                warn!("Webpage content matched (matched: {:?})", needle.chars().take(32).collect::<String>());
                true
            } else {
                false
            })
        );
    if has_illegal_text {
        return true;
    }

    // Audit any sketchy URLs from the webpage body as well
    if depth < MAX_DEPTH {
        if any_illegal_url(find_page_urls(&body), depth + 1).await {
            return true;
        }
    } else {
        warn!("Not scanning URLs on webpage conten, reached depth {}, assuming safe", depth);
    }

    false
}

/// Check wheher the given URL is illegal.
///
/// This checks the static URL, and does not do any redirect checking.
pub fn is_illegal_static_url(url: &Url) -> bool {
    // We must have illegal hosts or parts configured
    if ILLEGAL_HOSTS.is_empty() && ILLEGAL_HOST_PARTS.is_empty() {
        return false;
    }

    // Get the host
    let host = match url.host_str() {
        Some(host) => host,
        None => return false,
    };
    let host = host.trim().to_lowercase();

    // Match the URL against a list of banned hosts
    if ILLEGAL_HOSTS
        .iter()
        .any(|illegal_host| illegal_host == &host)
    {
        warn!("Found illegal host: {}", url);
        return true;
    }

    // Match the URL against a list of banned host parts
    let illegal = ILLEGAL_HOST_PARTS
        .iter()
        .any(|illegal_part| host.contains(illegal_part));
    if illegal {
        warn!("Found illegal host (contains illegal part): {}", url);
        return true;
    }

    debug!("Audited URL as safe: {}", url);
    false
}
