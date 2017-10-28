extern crate oparl_cache;
#[macro_use]
extern crate json;
extern crate reqwest;

mod common;

use common::*;

use oparl_cache::Cacher;

use reqwest::IntoUrl;

/// Assert that the cached server status method returns the correct list
#[test]
fn test_cached_server_list() {
    let url1 = "http://example1.com/".into_url().unwrap();
    let url2 = "http://example2.com/".into_url().unwrap();
    let storage = storage();

    assert_eq!(storage.get_cached_servers().unwrap(), vec![]);

    storage.cache(mocking_server(url1.clone())).unwrap();
    assert_eq!(storage.get_cached_servers().unwrap(), vec![url1.clone()]);

    storage.cache(mocking_server(url2.clone())).unwrap();
    assert_eq!(storage.get_cached_servers().unwrap(), vec![url1.clone(), url2.clone()]);

    storage.cache(mocking_server(url2.clone())).unwrap();
    assert_eq!(storage.get_cached_servers().unwrap(), vec![url1.clone(), url2.clone()]);

    cleanup(&storage);
}
