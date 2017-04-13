use reqwest::Url;

use std::path::Path;
use std::fs;

use oparl_cache::{FileStorage, DEFAULT_CACHE_STATUS_FILE};

use super::rand::Rng;
use super::rand;
use super::MockingServer;


#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn mocking_server(url: Url) -> MockingServer {
    let mut server = MockingServer::new(url.clone());
    server.add_response(url, object!{
        "id" => "http://example.com",
        "type" => "https://spec.oparl.org/1.0/System"
    });
    server
}


#[allow(dead_code)]
#[allow(unused_must_use)]
pub fn cleanup(storage: &FileStorage) {
    fs::remove_dir_all(storage.get_cache_dir());
}
