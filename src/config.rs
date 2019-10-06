/// A list of illegal URL hosts.
pub const ILLEGAL_HOSTS: [&str; 10] = [
    "binance.bnbnetwork.icu",
    "binance.webjersey.icu",
    "binance.jerseyonline.icu",
    "exchange.jerseysolution.services",
    "exchange.marketrelease.services",
    "binance.dexsupport.site",
    "binance.mxevent.site",
    "mxevent.site",
    "binance.com",
    "binance.us",
];

/// A list of illegal URL host parts.
pub const ILLEGAL_HOST_PARTS: [&str; 3] = ["binance", "jerseyonline", "jerseysolution"];

/// A list of illegal text.
pub const ILLEGAL_TEXT: [&str; 5] = [
    "Celebrating Our New Crypto Exchange",
    "Binance is pleased to announce the unmatched trading",
    "To celebrate the launch of Binance US",
    "Event ends today!",
    "First 5000 Participants Bonus",
    // "Вinаⴖce US",
    // "Βἱnаⴖcе US",
    // "𐌉МΡOR𐌕АΝΤ",
    // "ⵏMР𐩒RΤΑNТ",
    // "𐌏ⴖƖγ thе fіr𐑈t 5000 u𐑈егs wἱƖƖ be гewardеd",
    // "OnƖy the fἰгѕt 5000 u𐑈егѕ ԝіlƖ ƅe reԝаrdеd",
    // "ⴹνеⴖt еnd𐑈 tоԁау!",
    // "Ενеⴖt ends tоԁаγ!",
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
pub const MAX_FILE_SIZE: i64 = 10 * 1024 * 1024;

/// Images are illegal when their similarity to any template image is `<= threhold`.
pub const IMAGE_BAN_THRESHOLD: f64 = 0.5;
