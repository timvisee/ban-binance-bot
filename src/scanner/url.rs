use futures::prelude::*;
use url::Url;

use crate::{config::*, util};

/// Check whether the given text contains any illegal URLs.
///
/// This uses `ILLEGAL_HOSTS`.
pub async fn contains_illegal_urls(text: &str) -> Result<bool, ()> {
    // TODO: do a forwarding check, compare target URLs as well

    // Find URLs in the message
    let urls = util::url::find_urls(text);

    // Scan for any static illegal URLs in the text message
    let illegal = urls.iter().any(is_illegal_url);
    if illegal {
        return Ok(true);
    }

    // Resolve all URL redirects, and test for illegal URLs again
    // TODO: use iterator here
    for url in urls {
        // TODO: do not drop error here
        let url = util::url::follow_url(&url).inspect_err(|err| { dbg!(err); }).await;
        if let Ok(url) = &url {
            if is_illegal_url(url) {
                return Ok(true);
            }
        }
    }

    Ok(false)
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
