/// A list of illegal URL hosts.
pub const ILLEGAL_HOSTS: [&str; 5] = [
    "binance.dexsupport.site",
    "binance.mxevent.site",
    "mxevent.site",
    "binance.com",
    "binance.us",
];

/// The maximum file size in bytes of files to check for legality.
pub const MAX_FILE_SIZE: i64 = 2 * 1024 * 1024;

/// Images are illegal when their similarity to any template image is `<= threhold`.
pub const IMAGE_BAN_THRESHOLD: f64 = 0.07;
