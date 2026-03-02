pub const SITE_PATH: &str = "site";
pub const SOURCE_PATH: &str = "site/source";
pub const LIVE_PATH: &str = "site/live";

pub fn ensure_live_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(LIVE_PATH)
}
