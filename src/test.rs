use chrono::{Local};
use super::{OParlCache, ExternalList};
use std::path::Path;

#[test]
fn parse_object() {
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

    OParlCache::new().parse_object(&mut input);
    assert_eq!(input, expected_output);
}

#[test]
fn parse_external_list() {
    let eurl = "http://localhost:8080/oparl/v1.0/body/0/list/paper?a=b";
    let time = Local::now().format("%Y-%m-%dT%H:%M:%S%Z").to_string();
    OParlCache::new().parse_external_list(eurl, Some(time));
}

#[test]
fn external_list_iterator() {
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
    assert_eq! (OParlCache::new().url_to_path((url.to_string() + query_string).as_str(), ".json"), Path::new(path));
    assert_eq! (OParlCache::new().url_to_path((url.to_string() + "/" + query_string).as_str(), ".json"), Path::new(path));
}

#[test]
fn url_to_path() {
    single_url_to_path("https://example.tld:8080/oparl/v1.0/paper/1", "", "/home/konsti/cache-rust/https:example.tld:8080/oparl/v1.0/paper/1.json");
    single_url_to_path("https://example.tld/oparl/v1.0/paper/1", "", "/home/konsti/cache-rust/https:example.tld/oparl/v1.0/paper/1.json");
    single_url_to_path("https://example.tld/oparl/v1.0", "", "/home/konsti/cache-rust/https:example.tld/oparl/v1.0.json");
    single_url_to_path("https://example.tld", "", "/home/konsti/cache-rust/https:example.tld.json");
    single_url_to_path("https://example.tld/api", "?modified_until=2016-05-03T00%3A00%3A00%2B02%3A00", "/home/konsti/cache-rust/https:example.tld/api.json");
}