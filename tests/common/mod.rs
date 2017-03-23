extern crate reqwest;

use oparl_cache::{ FileStorage, DEFAULT_CACHE_STATUS_FILE, CommonServer};
use self::reqwest::IntoUrl;

pub fn storage<'a>() -> FileStorage<'a> {
    FileStorage::new("/home/konsti/oparl/schema",
                     "/tmp/cache-rust",
                     DEFAULT_CACHE_STATUS_FILE).unwrap()
}

pub fn server() -> CommonServer {
    CommonServer::new("http://localhost:8080".into_url().unwrap())
}

