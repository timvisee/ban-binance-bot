use std::time::Duration;

use reqwest::{r#async::Client, RedirectPolicy};
use url::Url;

use crate::{
    config::*,
    util::{self, future::select_true},
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

    // Test each URL concurrently
    select_true(urls.into_iter().map(is_illegal_url)).await
}

/// Check whether the given URL is illegal.
///
/// This compares the given URL, and the URL it possibly redirects to.
///
/// Returns `Ok` if the URL is illegal, `Err` otherwise.
/// Errors are silently dropped and it will then be assumed that the URL is allowed.
/// This allows the use of `futures::future::select_ok`.
async fn is_illegal_url(mut url: Url) -> bool {
    // The given URL must not be illegal
    if is_illegal_static_url(&url) {
        return true;
    }

    // Follow URL redirects
    match util::url::follow_url(&url).await {
        Ok(ref url) if is_illegal_static_url(url) => return true,
        Ok(new) => url = new,
        Err(err) => println!("failed to follow URL redirects to audit, ignoring: {:?}", err),
    }

    // Check whether the webpage contains illegal content
    if url_has_illegal_webpage_content(&url).await {
        println!("Webpage has illegal body! {}", url);
        return true;
    }

    false
}

/// Check whether the given URL routes to illegal content.
///
/// This scans the body of the webpage that is responded with.
async fn url_has_illegal_webpage_content(url: &Url) -> bool {
    // Skip if there are no filters
    if ILLEGAL_WEBPAGE_TEXT.is_empty() {
        return false;
    }

    // Build the URL client
    // TODO: use a global client instance
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(RedirectPolicy::limited(25))
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(20))
        .build()
        .expect("failed to build webpage body auditer client");

    // Send the request, follow the URL
    // TODO: validate status !response.status.is_success()
    let response = match client.get(url.as_str()).send().await {
        Ok(response) => response,
        Err(err) => {
            println!("failed to request webpage body bytes to audit, ignoring: {}", err);
            return false;
        },
    };

    // Request the page body
    let body = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(err) => {
            println!("failed to retrieve webpage body bytes to audit, ignoring: {}", err);
            return false;
        },
    };

    // Find the shortest needle to limit body searching
    let needles = ILLEGAL_WEBPAGE_TEXT;
    let shortest = needles.iter().map(|t| t.as_bytes().len()).min().unwrap();

    // Scan body for needles to detect illegal content
    (0..=body.len() - shortest)
        .any(|i| needles
            .iter()
            .filter(|needle| needle.as_bytes().len() <= body.len() - i)
            .any(|needle| &body[i..i + needle.len()] == needle.as_bytes())
        )
}

/// Check wheher the given URL is illegal.
///
/// This checks the static URL, and does not do any redirect checking.
pub fn is_illegal_static_url(url: &Url) -> bool {
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
        println!("Found illegal host! {}", url);
        return true;
    }

    // Match the URL against a list of banned host parts
    let illegal = ILLEGAL_HOST_PARTS
        .iter()
        .any(|illegal_part| host.contains(illegal_part));
    if illegal {
        println!("Found illegal host (contains illegal part)! {}", url);
        return true;
    }

    println!("Got legal url");
    false
}
