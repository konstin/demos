extern crate oparl_cache;
#[macro_use]
extern crate json;
extern crate reqwest;

mod common;

use common::*;

use oparl_cache::Cacher;

use reqwest::IntoUrl;

/// Tests the cached server status method
#[test]
fn test_cached_server_list() {
    let url1 = "http://example1.com/";
    let url2 = "http://example2.com/";
    let storage = storage();

    assert_eq!(storage.get_cached_servers().unwrap(), array![]);

    storage.cache(mocking_server(url1.into_url().unwrap())).unwrap();
    assert_eq!(storage.get_cached_servers().unwrap(), array![url1]);

    storage.cache(mocking_server(url2.into_url().unwrap())).unwrap();
    assert_eq!(storage.get_cached_servers().unwrap(), array![url1, url2]);

    storage.cache(mocking_server(url2.into_url().unwrap())).unwrap();
    assert_eq!(storage.get_cached_servers().unwrap(), array![url1, url2]);

    cleanup(&storage);
}
