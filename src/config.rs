/// A list of illegal URL hosts.
pub const ILLEGAL_HOSTS: [&str; 32] = [
    "binance.bnbnetwork.icu",
    "binance.bnbrelease.icu",
    "binance.channelevent.icu",
    "binance.dexexchange.icu",
    "binance.dexexchange.site",
    "binance.dexsupport.site",
    "binance.dexsupports.icu",
    "binance.eventonline.icu",
    "binance.exchangemarket.icu",
    "binance.jerseylaunch.site",
    "binance.jerseymx.site",
    "binance.jerseyonline.icu",
    "binance.jerseysolution.site",
    "binance.marketjersey.icu",
    "binance.marketrelease.icu",
    "binance.mxevent.site",
    "binance.webjersey.icu",
    "event.bnbexchange.services",
    "event.exchangelaunch.services",
    "exchange.2019event.top",
    "exchange.bnbdex.top",
    "exchange.bnblaunch.com",
    "exchange.bnblaunch.top",
    "exchange.bnbproject.services",
    "exchange.bnbsolutions.services",
    "exchange.channelevent.top",
    "exchange.dexmxjersey.services",
    "exchange.jerseymx.services",
    "exchange.jerseysolution.services",
    "exchange.marketrelease.services",
    "exchange.projectdex.services",
    "mxevent.site",
];

/// A list of illegal URL host parts.
pub const ILLEGAL_HOST_PARTS: [&str; 13] = [
    "binance.bnb",
    "binance.dex",
    "binance.event",
    "binance.exchange",
    "binance.jersey",
    "binance.market",
    "exchange.2019e",
    "exchange.bnb",
    "exchange.channelevent",
    "exchange.dexmx",
    "exchange.projectdex",
    "jerseyonline",
    "jerseysolution",
];

/// A list of illegal text.
pub const ILLEGAL_TEXT: [&str; 5] = [
    "Celebrating Our New Crypto Exchange",
    "Binance is pleased to announce the unmatched trading",
    "To celebrate the launch of Binance US",
    // "Event ends today!",
    "First 5000 Participants Bonus",
    "Only the first 5000 users will be rewarded",
];

/// A list of illegal text in webpage bodies.
pub const ILLEGAL_WEBPAGE_TEXT: [&str; 9] = [
    "First 5000 Participants BTC Giveaway!",
    "Celebrating the launch of our new Crypto Marketplace - Binance US",
    "We are pleased to announce the unmatched trading technology platform of Binance to the United States and all of North America",
    "To celebrate the launch of Binance US, we are rewarding the first 5000 participants with 10 times deposit bonus",
    "In order to be eligible, participants must have a minimum of 0.02 BTC",
    "Only the first 5000 users will be rewarded and it's on a first come first served basis. Qualifying users will receive the deposit bonus along with an invitation link to beta test Binance US. Every bug/hack/problem found on Binance US will be rewarded up to 10 BTC (more details upon sign-up).",
    "For every BTC contributed, you will receive back 10 times more BTC!",
    r#"<html><body><script type="text/javascript" src="/aes.js" ></script><script>function toNumbers(d){var e=[];d.replace(/(..)/g"#,
    r#"&i=1";</script><noscript>This site requires Javascript to work, please enable Javascript in your browser or use a browser with Javascript support</noscript></body></html>"#,
];

/// A list of hosts for URLs that should be scanned if appearing on the webpage.
pub const SCAN_WEBPAGE_URL_HOSTS: [&str; 4] = [
    "bit.ly",
    "t.cn",
    "t.co",
    "tinyurl.com",
];

/// A list of illegal text in images.
#[cfg(feature = "ocr")]
pub const ILLEGAL_IMAGE_TEXT: [&str; 3] = [
    "EVENT ENDS AT MIDNIGHT TODAY",
    "First 5000 Participants Bonus",
    "Catherine Coley",
];

/// Directory containing all illegal images.
pub const ILLEGAL_IMAGES_DIR: &str = "./res/illegal/";

/// The maximum file size in bytes of files to check for legality.
pub const MAX_FILE_SIZE: i64 = 100 * 1024 * 1024;

/// The maximum file size in bytes of images to check for legality.
pub const IMAGE_MAX_FILE_SIZE: i64 = 20 * 1024 * 1024;

/// Images are illegal when their similarity to any template image is `<= threhold`.
pub const IMAGE_BAN_THRESHOLD: f64 = 0.5;

/// The minimum number of pixels each image side must have.
///
/// This is for image matching. Image OCR will run on all images if enabled.
pub const IMAGE_MIN_SIZE: u32 = 80;

/// The max depth for scanning URLs.
///
/// This manages how deep this bot will go with scanning URLs on webpages recursively.
/// Following many URL redirects counts as 1 depth.
pub const MAX_DEPTH: usize = 4;

/// When auditing, compare images against banned database.
///
/// This is expensive when lots of images are listed as banned.
// TODO: this seems to leak memory when used a lot, investigate and fix, currently disabled
pub const AUDIT_IMAGE_COMPARE: bool = true;

/// Time after which to self-destruct ban notification messages by this bot.
///
/// Set to `None` to not self-destruct.
pub const NOTIFY_SELF_DESTRUCT_TIME: Option<u64> = Some(60);

lazy_static! {
    /// Number of Telegram API updates to process concurrently.
    pub static ref TELEGRAM_CONCURRENT_UPDATES: usize = num_cpus::get().max(2);

    /// Number of images to match at the same time.
    ///
    /// This is the maximum number of matches to run concurrently for each image against the list
    /// of banned image templates.
    pub static ref IMAGE_CONCURRENT_MATCHES: usize = num_cpus::get();
}
