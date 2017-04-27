use reqwest::Url;

use std::path::Path;
use std::fs;

use oparl_cache::{FileStorage};

use super::rand::Rng;
use super::rand::thread_rng;
use super::MockingServer;

#[allow(dead_code)]
pub fn storage<'a>() -> FileStorage<'a> {
    let uid = thread_rng()
        .gen_ascii_chars()
        .take(8)
        .collect::<String>();

    let path = Path::new("/tmp").join("cache-rust-".to_owned() + &uid);

    FileStorage::new(Path::new("oparl/schema"),
                     path)
            .unwrap()
}

/// Mocking Server with a stub System-object under `url`
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
