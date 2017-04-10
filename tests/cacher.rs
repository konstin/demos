extern crate oparl_cache;
#[macro_use]
extern crate json;
extern crate reqwest;

mod common;

use reqwest::{Url, IntoUrl};

use std::sync::mpsc::channel;

use oparl_cache::Cacher;
use oparl_cache::Message::{List, Done};
use oparl_cache::FILE_EXTENSION;
use oparl_cache::FileStorage;

use common::*;

#[test]
fn parse_object_and_extract_embedded_object() {
    let mut input = object! {
            "id" => "http://localhost:8080/oparl/v1.0/paper/2",
            "type" => "https://schema.oparl.org/1.0/Paper",
            "auxiliaryFile" => array![
                object!{
                  "id" => "http://localhost:8080/oparl/v1.0/file/2",
                  "type" => "https://schema.oparl.org/1.0/File",
                  "accessUrl" => "http://localhost:8080/fileaccess/access/2",
                  "created" => "2016-05-02T19:53:08+02:00",
                  "modified" => "2016-05-02T19:53:08+02:00"
                }
            ],
            "created" => "2016-05-02T00:00:00+02:00",
            "modified" => "2016-05-02T00:00:00+02:00"
        };

    // Create a deep copy and replace the embedded object by its id
    let mut expected_output = json::parse(&input.dump()).unwrap();
    expected_output["auxiliaryFile"][0] = expected_output["auxiliaryFile"][0]["id"].take();

    let storage = storage();
    let (add_list, receive_list) = channel();

    storage.parse_object(&mut input, add_list).unwrap();

    cleanup(&storage);

    assert_eq!(input, expected_output);
    assert_eq!(receive_list.recv().is_err(), true);
}

#[test]
fn parse_object_find_external_list() {
    let mut input = object! {
        "id" => "http://localhost:8080/oparl/v1.0/body/0",
        "type" => "https://schema.oparl.org/1.0/Body",
        "legislativeTerm" => array! [
            object! {
                "id" => "http://localhost:8080/oparl/v1.0/legislativeterm/0",
                "type" => "https://schema.oparl.org/1.0/LegislativeTerm",
                "name" => "Unbekannt"
            }
        ],
        "organization" => "http://localhost:8080/oparl/v1.0/body/0/list/organization",
        "person" => "http://localhost:8080/oparl/v1.0/body/0/list/person",
        "meeting" => "http://localhost:8080/oparl/v1.0/body/0/list/meeting",
        "paper" => "http://localhost:8080/oparl/v1.0/body/0/list/paper",
        "web" => "http://localhost:8080/",
        "created" => "2016-09-29T14:31:50+02:00",
        "modified" => "2016-09-29T14:42:52+02:00"
    };

    // Create a deep copy and replace the embedded object by its id
    let mut expected_output = json::parse(&input.dump()).unwrap();
    expected_output["legislativeTerm"][0] = expected_output["legislativeTerm"][0]["id"].take();

    let expected_lists =
        vec!["http://localhost:8080/oparl/v1.0/body/0/list/organization".into_url().unwrap(),
             "http://localhost:8080/oparl/v1.0/body/0/list/person".into_url().unwrap(),
             "http://localhost:8080/oparl/v1.0/body/0/list/meeting".into_url().unwrap(),
             "http://localhost:8080/oparl/v1.0/body/0/list/paper".into_url().unwrap()];

    let storage = storage();
    let (add_list, receive_list) = channel();

    storage.parse_object(&mut input, add_list).unwrap();

    cleanup(&storage);

    assert_eq!(input, expected_output);
    let results: Vec<Url> = receive_list.iter()
        .map(|url| match url {
                 List(url) => url,
                 Done => panic!(),
             })
        .collect();
    assert_eq!(results, expected_lists);
}


fn for_one(url: &str, query_string: &str, path: &str, storage: &FileStorage) {
    let x = (url.to_string() + query_string).into_url().unwrap();
    assert_eq!(storage.get_cache_dir().as_path().join(path), storage.url_to_path(&x, FILE_EXTENSION));
    let y = (url.to_string() + "/" + query_string).into_url().unwrap();
    assert_eq!(storage.get_cache_dir().as_path().join(path), storage.url_to_path(&y, FILE_EXTENSION));
}

#[test]
fn test_url_to_path() {
    let storage = storage();

    let cache_status_file = "http:localhost:8080/oparl/v1.0/cache-status.json";
    assert_eq!(storage.get_cache_dir().as_path().join(cache_status_file),
               storage
                   .url_to_path(&"http://localhost:8080/oparl/v1.0".into_url().unwrap(), "")
                   .join("cache-status.json"));
    for_one("https://example.tld:8080/oparl/v1.0/paper/1",
            "",
            "https:example.tld:8080/oparl/v1.0/paper/1.json",
            &storage);
    for_one("https://example.tld/oparl/v1.0/paper/1",
            "",
            "https:example.tld/oparl/v1.0/paper/1.json",
            &storage);
    for_one("https://example.tld/oparl/v1.0",
            "",
            "https:example.tld/oparl/v1.0.json",
            &storage);
    for_one("https://example.tld",
            "",
            "https:example.tld/.json",
            &storage);
    for_one("https://example.tld/api",
            "?modified_until=2016-05-03T00%3A00%3A00%2B02%3A00",
            "https:example.tld/api.json",
            &storage);

    cleanup(&storage);

}
