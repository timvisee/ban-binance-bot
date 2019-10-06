use futures::prelude::*;
use url::Url;

use crate::{config::*, util};

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
    let test_urls = urls.into_iter().map(|u| is_illegal_url(u).boxed());
    futures::future::select_ok(test_urls).await.is_ok()
}

/// Check whether the given URL is illegal.
///
/// This compares the given URL, and the URL it possibly redirects to.
///
/// Returns `Ok` if the URL is illegal, `Err` otherwise.
/// Errors are silently dropped and it will then be assumed that the URL is allowed.
/// This allows the use of `futures::future::select_ok`.
async fn is_illegal_url(url: Url) -> Result<(), ()> {
    // The given URL must not be illegal
    if is_illegal_static_url(&url) {
        return Ok(());
    }

    // Follow URL redirects
    match util::url::follow_url(&url).await {
        Ok(ref url) if is_illegal_static_url(url) => Ok(()),
        Ok(_) => Err(()),
        Err(err) => {
            // TODO: do not drop error here
            dbg!(err);
            Err(())
        }
    }
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
        println!("Found illegal host (on illegal host)!");
        return true;
    }

    // Match the URL against a list of banned host parts
    let illegal = ILLEGAL_HOST_PARTS
        .iter()
        .any(|illegal_part| host.contains(illegal_part));
    if illegal {
        println!("Found illegal host (contains illegal part)!");
        return true;
    }

    println!("Got legal url");
    false
}
