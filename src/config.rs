/// A list of illegal URL hosts.
pub const ILLEGAL_HOSTS: [&str; 16] = [
    "binance.bnbnetwork.icu",
    "binance.dexexchange.icu",
    "binance.dexsupport.site",
    "binance.eventonline.icu",
    "binance.jerseymx.site",
    "binance.jerseyonline.icu",
    "binance.mxevent.site",
    "binance.us",
    "binance.webjersey.icu",
    "event.exchangelaunch.services",
    "exchange.bnblaunch.com",
    "exchange.bnblaunch.top",
    "exchange.bnbsolutions.services",
    "exchange.jerseysolution.services",
    "exchange.marketrelease.services",
    "mxevent.site",
];

/// A list of illegal URL host parts.
pub const ILLEGAL_HOST_PARTS: [&str; 6] = [
    "binance.dex",
    "binance.event",
    "binance.exchange",
    "binance.jersey",
    "jerseyonline",
    "jerseysolution",
];

/// A list of illegal text.
pub const ILLEGAL_TEXT: [&str; 6] = [
    "Celebrating Our New Crypto Exchange",
    "Binance is pleased to announce the unmatched trading",
    "To celebrate the launch of Binance US",
    "Event ends today!",
    "First 5000 Participants Bonus",
    "Only the first 5000 users will be rewarded",
    // "Вinаⴖce US",
    // "Βἱnаⴖcе US",
    // "𐌉МΡOR𐌕АΝΤ",
    // "ⵏMР𐩒RΤΑNТ",
    // "𐌏ⴖƖγ thе fіr𐑈t 5000 u𐑈егs wἱƖƖ be гewardеd",
    // "OnƖy the fἰгѕt 5000 u𐑈егѕ ԝіlƖ ƅe reԝаrdеd",
    // "ⴹνеⴖt еnd𐑈 tоԁау!",
    // "Ενеⴖt ends tоԁаγ!",
];

/// A list of illegal text in webpage bodies.
pub const ILLEGAL_WEBPAGE_TEXT: [&str; 7] = [
    "First 5000 Participants BTC Giveaway!",
    "Celebrating the launch of our new Crypto Marketplace - Binance US",
    "We are pleased to announce the unmatched trading technology platform of Binance to the United States and all of North America",
    "To celebrate the launch of Binance US, we are rewarding the first 5000 participants with 10 times deposit bonus",
    "In order to be eligible, participants must have a minimum of 0.02 BTC",
    "Only the first 5000 users will be rewarded and it's on a first come first served basis. Qualifying users will receive the deposit bonus along with an invitation link to beta test Binance US. Every bug/hack/problem found on Binance US will be rewarded up to 10 BTC (more details upon sign-up).",
    "For every BTC contributed, you will receive back 10 times more BTC!",
];

/// A list of illegal text in images.
#[cfg(feature = "ocr")]
pub const ILLEGAL_IMAGE_TEXT: [&str; 4] = [
    "EVENT ENDS AT MIDNIGHT TODAY",
    "First 5000 Participants Bonus",
    "Catherine Coley",
    "Binance US",
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

lazy_static! {
    /// Number of Telegram API updates to process concurrently.
    pub static ref TELEGRAM_CONCURRENT_UPDATES: usize = num_cpus::get().max(2);

    /// Number of images to match at the same time.
    ///
    /// This is the maximum number of matches to run concurrently for each image against the list
    /// of banned image templates.
    pub static ref IMAGE_CONCURRENT_MATCHES: usize = num_cpus::get();
}
