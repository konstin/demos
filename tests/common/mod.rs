extern crate reqwest;
use std::path::Path;

use oparl_cache::{FileStorage, DEFAULT_CACHE_STATUS_FILE, CommonServer};
use self::reqwest::IntoUrl;

pub fn storage<'a>() -> FileStorage<'a> {
    let uid = rand::thread_rng()
        .gen_ascii_chars()
        .take(8)
        .collect::<String>();

    let path = Path::new("/tmp").join("cache-rust-".to_owned() + &uid);

    FileStorage::new(Path::new("/home/konsti/oparl/schema"),
                     path,
                     DEFAULT_CACHE_STATUS_FILE)
            .unwrap()
}

pub fn server() -> CommonServer {
    CommonServer::new("http://localhost:8080".into_url().unwrap())
}
