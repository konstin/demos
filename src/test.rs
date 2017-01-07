use super::*;

use hyper::client::IntoUrl;

/// Helper for various tests
pub fn storage<'a>() -> Storage<'a> {
    Storage::new(
        "http://localhost:8080/oparl/v1.0".into_url().unwrap(),
        "/home/konsti/oparl/schema/",
        "/home/konsti/cache-rust/",
        DEFAULT_CACHE_STATUS_FILE
    ).unwrap()
}


