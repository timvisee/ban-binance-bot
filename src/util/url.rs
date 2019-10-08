use std::time::Duration;

use regex::Regex;
use reqwest::{r#async::Client, Error as ResponseError, RedirectPolicy};
use url::Url;

lazy_static! {
    // A regex for detecting URLs.
    static ref URL_REGEX: Regex = Regex::new(r"(?:(?:https?|ftp)://)?(?:\S+(?::\S*)?@|\d{1,3}(?:\.\d{1,3}){3}|(?:(?:[a-z\d\x{00a1}-\x{ffff}]+-?)*[a-z\d\x{00a1}-\x{ffff}]+)(?:\.(?:[a-z\d\x{00a1}-\x{ffff}]+-?)*[a-z\d\x{00a1}-\x{ffff}]+)*(?:\.[a-z\x{00a1}-\x{ffff}]{2,6}))(?::\d+)?(?:[^\s]*)?")
        .expect("failed to compile URL regex");
}

/// List all URLs in the given text.
pub fn find_urls(text: &str) -> Vec<Url> {
    // Collect all links, parse them to URL
    URL_REGEX.find_iter(text)
        .map(|url| {
            // Prefix protocol if not set
            let mut url = url.as_str().trim().to_owned();
            if !url.starts_with("http") && !url.starts_with("ftp") {
                url.insert_str(0, "https://");
            }
            url
        })
        .filter_map(|url| match Url::parse(url.as_str()) {
            Ok(url) => Some(url),
            Err(err) => {
                eprintln!("Failed to parse URL: {:?}", err);
                None
            }
        })
        .collect()
}

/// Follow redirects on the given URL, and return the final full URL.
///
/// This is used to obtain share URLs from shortened links.
pub async fn follow_url(url: &Url) -> Result<Url, FollowError> {
    // Build the URL client
    // TODO: use a global client instance
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(RedirectPolicy::limited(25))
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(20))
        .build()
        .expect("failed to build URL forward resolver client");

    println!("Checking URL for redirects: {}", url.as_str());

    // Send the request, follow the URL, return target URL or last known URL from error
    // TODO: validate status !response.status.is_success()
    match client.get(url.as_str()).send().await {
        Ok(response) => Ok(response.url().clone()),
        Err(err) => err.url().cloned().ok_or(FollowError::Request(err)),
    }
}

/// URL following error.
#[derive(Debug)]
pub enum FollowError {
    /// Failed to send the shortening request.
    // #[fail(display = "failed to send URL follow request")]
    Request(reqwest::Error),

    /// The server responded with a bad response.
    // #[fail(display = "failed to shorten URL, got bad response")]
    Response(ResponseError),
}

impl From<ResponseError> for FollowError {
    fn from(err: ResponseError) -> Self {
        FollowError::Response(err)
    }
}
