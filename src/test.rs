use super::*;

use reqwest::client::IntoUrl;

/// Helper for various tests
pub fn storage<'a>() -> FileStorage<'a> {
    FileStorage::new("/home/konsti/oparl/schema/",
                     "/tmp/cache-rust/",
                     DEFAULT_CACHE_STATUS_FILE)
            .unwrap()
}
