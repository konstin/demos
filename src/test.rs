use chrono::{Local};
use json;

use std::path::Path;
use std::sync::{Arc, Mutex};

use super::OParlCache;
use super::external_list::ExternalList;

fn instance<'a>() -> OParlCache<'a> {
    OParlCache::new(
        "http://localhost:8080/oparl/v1.0",
        "/home/konsti/oparl/schema/",
        "/home/konsti/cache-rust/"
    )
}

#[test]
fn parse_object_extract_internal() {
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

    let expected_output = object! {
        "id" => "http://localhost:8080/oparl/v1.0/paper/2",
        "type" => "https://schema.oparl.org/1.0/Paper",
        "auxiliaryFile" => array! [
            "http://localhost:8080/oparl/v1.0/file/2"
        ],
        "created" => "2016-05-02T00:00:00+02:00",
        "modified" => "2016-05-02T00:00:00+02:00"
    };

    let external_list_adder = Arc::new(Mutex::new(Vec::new()));
    instance().parse_object(&mut input, &external_list_adder);
    assert_eq!(input, expected_output);
    assert_eq!(*external_list_adder.lock().unwrap(), vec![]);
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

    let external_list_adder = Arc::new(Mutex::new(Vec::new()));
    instance().parse_object(&mut input, &external_list_adder);

    assert_eq!(input, expected_output);
    assert_eq!(*external_list_adder.lock().unwrap(), vec![
        ("http://localhost:8080/oparl/v1.0/body/0/list/organization".to_string(), None),
        ("http://localhost:8080/oparl/v1.0/body/0/list/person".to_string(), None),
        ("http://localhost:8080/oparl/v1.0/body/0/list/meeting".to_string(), None),
        ("http://localhost:8080/oparl/v1.0/body/0/list/paper".to_string(), None),
    ]);
}

#[test]
fn parse_external_list() {
    let eurl = "http://localhost:8080/oparl/v1.0/body/0/list/paper";
    let time = Local::now().format("%Y-%m-%dT%H:%M:%S%Z").to_string();
    let external_list_adder = Arc::new(Mutex::new(Vec::new()));
    instance().parse_external_list(eurl, Some(time), &external_list_adder);
    assert_eq!(*external_list_adder.lock().unwrap(), vec![]);
}

#[test]
fn external_list() {
    let expected_ids = [
        "http://localhost:8080/oparl/v1.0/paper/1",
        "http://localhost:8080/oparl/v1.0/paper/2",
        "http://localhost:8080/oparl/v1.0/paper/3",
        "http://localhost:8080/oparl/v1.0/paper/4",
        "http://localhost:8080/oparl/v1.0/paper/5",
        "http://localhost:8080/oparl/v1.0/paper/6",
        "http://localhost:8080/oparl/v1.0/paper/7",
    ];

    let eurl = "http://localhost:8080/oparl/v1.0/body/0/list/paper";
    let list = ExternalList::new(eurl.to_string());
    let ids = list.map(|i| i["id"].to_owned()).collect::<Vec<_>>();
    assert_eq!(ids, expected_ids);
}


fn single_url_to_path(url: &str, query_string: &str, path: &str) {
    assert_eq! (instance().url_to_path((url.to_string() + query_string).as_str(), ".json"), Path::new(path));
    assert_eq! (instance().url_to_path((url.to_string() + "/" + query_string).as_str(), ".json"), Path::new(path));
}

#[test]
fn url_to_path() {
    let cache_status_file = "/home/konsti/cache-rust/http:localhost:8080/oparl/v1.0/cache-status.json";
    assert_eq! (instance().url_to_path(instance().entrypoint, "").join("cache-status.json"), Path::new(cache_status_file));

    single_url_to_path("https://example.tld:8080/oparl/v1.0/paper/1", "", "/home/konsti/cache-rust/https:example.tld:8080/oparl/v1.0/paper/1.json");
    single_url_to_path("https://example.tld/oparl/v1.0/paper/1", "", "/home/konsti/cache-rust/https:example.tld/oparl/v1.0/paper/1.json");
    single_url_to_path("https://example.tld/oparl/v1.0", "", "/home/konsti/cache-rust/https:example.tld/oparl/v1.0.json");
    single_url_to_path("https://example.tld", "", "/home/konsti/cache-rust/https:example.tld.json");
    single_url_to_path("https://example.tld/api", "?modified_until=2016-05-03T00%3A00%3A00%2B02%3A00", "/home/konsti/cache-rust/https:example.tld/api.json");
}