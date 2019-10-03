use futures::{future::ok, stream::iter_ok, Future, Stream};
use url::Url;

use crate::{config::*, util};

/// Check whether the given text contains any illegal URLs.
///
/// This uses `ILLEGAL_HOSTS`.
pub fn contains_illegal_urls(text: &str) -> Box<dyn Future<Item = bool, Error = ()>> {
    // TODO: do a forwarding check, compare target URLs as well

    // Find URLs in the message
    let urls = util::url::find_urls(text);

    // Scan for any static illegal URLs in the text message
    let illegal = urls.iter().any(is_illegal_url);
    if illegal {
        return Box::new(ok(true));
    }

    // Resolve all URL forwards, and test for illegal URLs again
    let future = iter_ok(urls)
        // Filter URLs that are still the same
        .and_then(|url| util::url::follow_url(&url))
        .and_then(|url| ok(is_illegal_url(&url)))
        // TODO: do not map errors here
        .into_future()
        // TODO: test all results here
        .map(|(result, _)| result.unwrap_or(false))
        .map_err(|_| ());

    // Follow redirects on all URLs, and test the target URLs again
    Box::new(future)
}

/// Check wheher the given URL is illegal.
pub fn is_illegal_url(url: &Url) -> bool {
    // Get the host
    let host = match url.host_str() {
        Some(host) => host,
        None => return false,
    };
    let host = host.trim().to_lowercase();

    // Match the URL against a list of banned hosts
    let illegal = ILLEGAL_HOSTS
        .iter()
        .any(|illegal_host| illegal_host == &host);
    if illegal {
        println!("Found illegal host!");
    }
    illegal
}
