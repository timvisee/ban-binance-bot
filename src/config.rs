use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: General,
    pub scanner: Scanner,
}

impl Config {
    /// Load the configuration from the given path.
    pub fn from_path(path: &str) -> Result<Self, Error> {
        toml::from_str(&fs::read_to_string(path).map_err(Error::Read)?).map_err(Error::Toml)
    }
}

#[derive(Debug)]
pub enum Error {
    /// Failed to read configuration from disk.
    Read(std::io::Error),

    /// Toml format error.
    Toml(toml::de::Error),
}

#[derive(Debug, Deserialize)]
pub struct General {
    pub notification_self_destruct: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct Scanner {
    pub text: Text,
    pub web: Web,
    pub image: Image,
}

#[derive(Debug, Deserialize)]
pub struct Text {
    pub text: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Web {
    pub hosts: Vec<String>,
    pub host_parts: Vec<String>,
    pub text: Vec<String>,
}

// TODO: do not allow cloning this, use references
#[derive(Debug, Deserialize, Clone)]
pub struct Image {
    // TODO: change to PathBuf?
    pub dir: Option<String>,
    pub threshold: f32,
    pub text: Vec<String>,
}

/// A list of hosts for URLs that should be scanned if appearing on the webpage.
pub const SCAN_WEBPAGE_URL_HOSTS: [&str; 4] = ["bit.ly", "t.cn", "t.co", "tinyurl.com"];

/// The maximum file size in bytes of files to check for legality.
pub const MAX_FILE_SIZE: i64 = 100 * 1024 * 1024;

/// The maximum file size in bytes of images to check for legality.
pub const IMAGE_MAX_FILE_SIZE: i64 = 20 * 1024 * 1024;

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
