use std::time::Duration;

use futures::prelude::*;
use linkify::{LinkFinder, LinkKind};
use reqwest::{r#async::Client, Error as ResponseError, RedirectPolicy};
use url::Url;

/// List all URLs in the given text.
pub fn find_urls(text: &str) -> Vec<Url> {
    // Set up the URL finder
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url]);

    // Collect all links, parse them to URL
    finder
        .links(text)
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

    // Send the request, follow the URL
    let response = client
        .get(url.as_str())
        .send()
        .await;

    // TODO: validate status !response.status.is_success()

    // Return the target URL, or the last known URL on error
    match response {
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
