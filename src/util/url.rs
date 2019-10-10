use std::time::Duration;

use regex::Regex;
use reqwest::{r#async::Client, Error as ResponseError, RedirectPolicy};
use telegram_bot::types::{MessageEntity, MessageEntityKind};
use url::Url;

lazy_static! {
    // A regex for detecting URLs.
    static ref URL_REGEX: Regex = Regex::new(
        r"(?:(?:https?|ftp)://)?(?:\S+(?::\S*)?@|\d{1,3}(?:\.\d{1,3}){3}|(?:(?:[a-z\d\x{00a1}-\x{ffff}]+-?)*[a-z\d\x{00a1}-\x{ffff}]+)(?:\.(?:[a-z\d\x{00a1}-\x{ffff}]+-?)*[a-z\d\x{00a1}-\x{ffff}]+)*(?:\.[a-z\x{00a1}-\x{ffff}]{2,6}))(?::\d+)?(?:[^\s]*)?",
    ).expect("failed to compile URL regex");
}

/// List all URLs in the given text.
pub fn find_urls(text: &str) -> Vec<Url> {
    // Collect all links, parse them to URL
    URL_REGEX.find_iter(text)
        .map(|url| {
            // Prefix protocol if not set
            // TODO: do not trim suffixed ), remove when this issue is resolved
            // Issue: https://github.com/robinst/linkify/issues/7
            let mut url = url.as_str().trim().trim_end_matches(')').to_owned();
            if !url.starts_with("http") && !url.starts_with("ftp") {
                url.insert_str(0, "https://");
            }
            url
        })
        // TODO: remove this filter once proper URL checking is implemented in reqwest
        // Issue: https://github.com/seanmonstar/reqwest/issues/668
        .filter(|url| match url.parse::<hyper::Uri>() {
            Ok(_) => true,
            Err(err) => {
                warn!("Failed to parse URL '{}' as URI: {}", url, err);
                false
            }
        })
        .filter_map(|url| match Url::parse(url.as_str()) {
            Ok(url) => Some(url),
            Err(err) => {
                warn!("Failed to parse URL: {}", err);
                None
            }
        })
        .collect()
}

/// Find all URLs in a message that are normally hidden in text.
///
/// The `entities` for the Telegram message must be given.
pub fn find_hidden_urls(entities: &[MessageEntity]) -> Vec<Url> {
    entities
        .iter()
        .filter_map(|entity| match entity.kind {
            MessageEntityKind::TextLink(ref url) => Some(url),
            _ => None,
        })
        .filter_map(|url| match Url::parse(url.as_str()) {
            Ok(url) => Some(url),
            Err(err) => {
                warn!("Failed to parse URL: {}", err);
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

    debug!("Test URL for redirects: {}", url.as_str());

    // Send request to URL, get last known URL
    // TODO: validate status !response.status.is_success()
    let url = match client.get(url.as_str()).send().await {
        Ok(response) => Ok(response.url().clone()),
        Err(err) => err.url().cloned().ok_or(FollowError::Request(err)),
    };

    if let Ok(url) = &url {
        trace!("Url lead to: {}", url);
    }

    url
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
